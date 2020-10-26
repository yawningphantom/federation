use crate::request_pipeline::service_definition::{Service, ServiceDefinition};
use crate::transports::http::{GraphQLResponse, RequestContext};
use crate::utilities::deep_merge::merge;
use crate::utilities::json::JsonSliceValue;
use crate::Result;
use apollo_query_planner::model::Selection::Field;
use apollo_query_planner::model::Selection::InlineFragment;
use apollo_query_planner::model::*;
use async_lock::RwLock;
use futures::future::{BoxFuture, FutureExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{instrument, trace};

pub struct ExecutionContext<'schema, 'req> {
    service_map: &'schema HashMap<String, ServiceDefinition>,
    request_context: &'req RequestContext<'req>,
}

#[instrument(skip(query_plan, service_map, request_context))]
pub async fn execute_query_plan<'req>(
    query_plan: &QueryPlan,
    service_map: &HashMap<String, ServiceDefinition>,
    request_context: &'req RequestContext<'req>,
) -> Result<GraphQLResponse> {
    let context = ExecutionContext {
        service_map,
        request_context,
    };

    let data_lock: RwLock<GraphQLResponse> = RwLock::new(GraphQLResponse::default());
    trace!("QueryPlan: {}", serde_json::to_string(query_plan).unwrap());

    if let Some(ref node) = query_plan.node {
        execute_node(&context, node, &data_lock, &vec![]).await;
    } else {
        unimplemented!("Introspection not supported yet");
    };

    let data = data_lock.into_inner();
    Ok(data)
}

fn execute_node<'schema, 'req>(
    context: &'req ExecutionContext<'schema, 'req>,
    node: &'req PlanNode,
    response_lock: &'req RwLock<GraphQLResponse>,
    path: &'req ResponsePath,
) -> BoxFuture<'req, ()> {
    async move {
        match node {
            PlanNode::Fetch(fetch_node) => {
                let result = execute_fetch(context, &fetch_node, response_lock).await;
                if let Err(_e) = result {
                    unimplemented!("Handle error")
                }
            }
            PlanNode::Flatten(flatten_node) => {
                let mut flattend_path = Vec::from(path.as_slice());
                flattend_path.extend_from_slice(flatten_node.path.as_slice());

                let inner_lock: RwLock<GraphQLResponse> = RwLock::new(GraphQLResponse::default());

                /*
                    Flatten works by selecting a zip of the result tree from the
                    path on the node (i.e [topProducts, @]) and creating a temporary
                    RwLock JSON object for the data currently stored there. Then we proceed
                    with executing the result of the node tree in the plan. Once the nodes have
                    been executed, we restitch the temporary JSON back into the parent result tree
                    at the same point using the flatten path

                    results_to_flatten = {
                        topProducts: [
                            { __typename: "Book", isbn: "1234" }
                        ]
                    }

                    inner_to_merge = {
                        { __typename: "Book", isbn: "1234" }
                    }
                */
                {
                    let response_read_guard = response_lock.read().await;
                    let slice = JsonSliceValue::from_path_and_value(
                        &flatten_node.path,
                        &response_read_guard.data,
                    );

                    let mut inner_response_write_guard = inner_lock.write().await;
                    *inner_response_write_guard = GraphQLResponse {
                        data: slice.into_value(),
                        errors: None,
                    };
                }

                execute_node(context, &flatten_node.node, &inner_lock, &flattend_path).await;

                // once the node has been executed, we need to restitch it back to the parent
                // node on the tree of result data
                /*
                    results_to_flatten = {
                        topProducts: []
                    }

                    inner_to_merge = {
                        { __typename: "Book", isbn: "1234", name: "Best book ever" }
                    }

                    path = [topProducts, @]
                */
                {
                    let mut response_write_guard = response_lock.write().await;
                    let sliced_response = inner_lock.into_inner();
                    merge_flattend_responses(
                        &mut *response_write_guard,
                        sliced_response,
                        &flatten_node.path,
                    );
                }
            }
            PlanNode::Sequence { nodes } => {
                for node in nodes {
                    execute_node(context, &node, response_lock, path).await;
                }
            }
            PlanNode::Parallel { nodes } => {
                let mut promises = vec![];

                for node in nodes {
                    promises.push(execute_node(context, &node, response_lock, path));
                }
                futures::future::join_all(promises).await;
            }
        }
    }
    .boxed()
}

fn merge_flattend_responses(
    parent_response: &mut GraphQLResponse,
    child_response: GraphQLResponse,
    path: &[String],
) {
    if let Some(child_errors) = child_response.errors {
        if let Some(ref mut parent_errors) = parent_response.errors {
            parent_errors.extend(child_errors.into_iter())
        } else {
            parent_response.errors = Some(child_errors)
        }
    }

    fn merge_data(parent_data: &mut Value, child_data: &Value, path: &[String]) {
        if path.is_empty() || child_data.is_null() {
            merge(&mut *parent_data, &child_data);
            return;
        }

        if let Some((current, rest)) = path.split_first() {
            if current == "@" {
                if parent_data.is_array() && child_data.is_array() {
                    let parent_array = parent_data.as_array_mut().unwrap();
                    for index in 0..parent_array.len() {
                        if let Some(child_item) = child_data.get(index) {
                            let parent_item = parent_data.get_mut(index).unwrap();
                            merge_data(parent_item, child_item, &rest.to_owned());
                        }
                    }
                }
            } else if parent_data.get(&current).is_some() {
                let inner: &mut Value = parent_data.get_mut(&current).unwrap();
                merge_data(inner, child_data, &rest.to_owned());
            }
        }
    }

    merge_data(&mut parent_response.data, &child_response.data, path)
}

async fn execute_fetch<'schema, 'req>(
    context: &ExecutionContext<'schema, 'req>,
    fetch: &FetchNode,
    response_lock: &'req RwLock<GraphQLResponse>,
) -> Result<()> {
    let service = &context.service_map[&fetch.service_name];

    let mut variables: HashMap<String, Value> = HashMap::new();
    if !fetch.variable_usages.is_empty() {
        for variable_name in &fetch.variable_usages {
            if let Some(vars) = &context.request_context.graphql_request.variables {
                if let Some(variable) = vars.get(&variable_name) {
                    variables.insert(variable_name.to_string(), variable.clone());
                }
            }
        }
    }

    let mut representations_to_entity: Vec<usize> = vec![];

    let variables = if let Some(requires) = fetch.requires.as_ref() {
        if variables.contains_key("representations") {
            panic!("variables must not contain key named 'represenations'");
        }

        update_variables_with_representations(
            variables,
            response_lock,
            requires,
            &mut representations_to_entity,
        )
        .await
    } else {
        variables
    };

    let data_received = service
        .send_operation(context.request_context, fetch.operation.clone(), variables)
        .await?;

    if fetch.requires.is_some() {
        if let Some(recieved_entities) = data_received.get("_entities") {
            let mut entities_to_merge = response_lock.write().await;
            let data = &mut (*entities_to_merge).data;
            trace!(
                "{{\"merge\": {}, \"into entities\": {}, \"with indexes\": {:?}}}",
                serde_json::to_string(recieved_entities).unwrap(),
                serde_json::to_string(data).unwrap(),
                representations_to_entity
            );
            match data {
                Value::Array(entities) => {
                    for (repr_idx, entity_idx) in representations_to_entity.into_iter().enumerate()
                    {
                        if let Some(entity) = entities.get_mut(entity_idx) {
                            merge(entity, &recieved_entities[repr_idx]);
                        }
                    }
                }
                Value::Object(_) => {
                    merge(data, &recieved_entities[0]);
                }
                _ => {}
            }
        } else {
            panic!("Expexected data._entities to contain elements");
        }
    } else {
        let mut results_to_merge = response_lock.write().await;
        merge(&mut (*results_to_merge).data, &data_received);
    }

    Ok(())
}

fn execute_selection_set(source: &Value, selections: &SelectionSet) -> Value {
    if source.is_null() {
        return Value::default();
    }

    let mut result: Value = json!({});

    for selection in selections {
        match selection {
            Field(field) => {
                let response_name = field.alias.as_ref().unwrap_or(&field.name);

                if let Some(response_value) = source.get(response_name) {
                    if let Value::Array(inner) = response_value {
                        result[response_name] = Value::Array(
                            inner
                                .iter()
                                .map(|element| {
                                    if field.selections.is_some() {
                                        // TODO(ran) FIXME: QQQ Should this be `field.selections` ?
                                        execute_selection_set(element, selections)
                                    } else {
                                        element.clone()
                                    }
                                })
                                .collect(),
                        );
                    } else if let Some(ref selections) = field.selections {
                        result[response_name] = execute_selection_set(response_value, selections);
                    } else {
                        result[response_name] = serde_json::to_value(response_value).unwrap();
                    }
                } else {
                    panic!(
                        "Field '{}' was not found in response {}",
                        response_name,
                        serde_json::to_string(source).unwrap()
                    );
                }
            }
            InlineFragment(fragment) => {
                // TODO(ran) FIXME: QQQ if there's no type_condition, we don't recurse?
                if let Some(ref type_condition) = fragment.type_condition {
                    if let Some(typename) = source.get("__typename") {
                        if typename.as_str().unwrap() == type_condition {
                            merge(
                                &mut result,
                                &execute_selection_set(source, &fragment.selections),
                            );
                        }
                    }
                }
            }
        }
    }

    result
}

async fn update_variables_with_representations(
    mut variables: HashMap<String, Value>,
    response: &RwLock<GraphQLResponse>,
    requires: &SelectionSet,
    representations_to_entity: &mut Vec<usize>,
) -> HashMap<String, Value> {
    let mut representations: Vec<Value> = vec![];

    let mut update_with_entity = |idx: usize, entity: &Value| {
        let representation = execute_selection_set(entity, requires);
        if representation.is_object() && representation.get("__typename").is_some() {
            representations.push(representation);
            representations_to_entity.push(idx);
        }
    };

    let read_guard = response.read().await;
    let data = &read_guard.data;

    match data {
        Value::Array(entities) => {
            for (index, entity) in entities.iter().enumerate() {
                update_with_entity(index, entity);
            }
        }
        entity @ Value::Object(_) => {
            update_with_entity(0, entity);
        }
        v => unreachable!("`data` can only be an object or an array, data: {}", v),
    };

    variables.insert(
        String::from("representations"),
        Value::Array(representations),
    );
    variables
}

// TODO(ran) FIXME: update message on various unreachable!s
// TODO(ran) FIXME: replace panics with Error
