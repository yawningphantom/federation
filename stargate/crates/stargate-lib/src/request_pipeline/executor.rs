use crate::request_pipeline::service_definition::{Service, ServiceDefinition};
use crate::transports::http::{GraphQLResponse, RequestContext};
use crate::utilities::deep_merge::{merge, merge2};
use crate::utilities::json::{JsonSliceValue, JsonSliceValueMut};
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
    trace!(
        "QueryPlan: {}",
        serde_json::to_string(query_plan).expect("QueryPlan must serde")
    );

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
                let result = execute_fetch(context, fetch_node, response_lock).await;
                if let Err(_e) = result {
                    unimplemented!("Handle fetch error")
                }
            }
            PlanNode::Flatten(flatten_node) => {
                let result = execute_flatten(context, flatten_node, response_lock).await;
                if let Err(_e) = result {
                    unimplemented!("Handle flatten error")
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

async fn execute_flatten<'schema, 'req>(
    context: &ExecutionContext<'schema, 'req>,
    flatten_node: &'req FlattenNode,
    response_lock: &'req RwLock<GraphQLResponse>,
) -> Result<()> {
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
    let inner_response: RwLock<GraphQLResponse> = {
        let response_read_guard = response_lock.read().await;
        let slice =
            JsonSliceValue::from_path_and_value(&flatten_node.path, &response_read_guard.data);

        RwLock::new(GraphQLResponse {
            data: slice.into_value(),
            errors: None,
        })
    };

    if let PlanNode::Fetch(fetch) = &flatten_node.node.as_ref() {
        let result = execute_entities_fetch(context, fetch, &inner_response).await;
        if let Err(_e) = result {
            unimplemented!("Handle error")
        }
    } else {
        panic!("The node in a Flatten node is always a Fetch node")
    }

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
        let mut parent_response_guard = response_lock.write().await;
        let child_response = inner_response.into_inner();
        merge_flattend_responses(
            &mut *parent_response_guard,
            child_response,
            &flatten_node.path,
        );
    }

    Ok(())
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

    let parent_data = &mut parent_response.data;
    let child_data = child_response.data;

    if child_data.is_null() {
        // Nothing to do
        return;
    }

    let slice = JsonSliceValueMut::new(parent_data).slice_by_path(path);

    let child_data = match child_data {
        Value::Array(child_data) => child_data,
        v @ Value::Object(_) => vec![v],
        _ => unreachable!(
            "child_response is the result of an entities fetch that returns an array or object"
        ),
    };

    for (parent, child) in slice.flatten().into_iter().zip(child_data.into_iter()) {
        match parent {
            JsonSliceValueMut::Value(parent) => merge2(parent, child),
            JsonSliceValueMut::Null => (),
            JsonSliceValueMut::Array(_) => {
                unreachable!("the slice has been flattened out of all arrays")
            }
        }
    }
}

async fn execute_fetch<'schema, 'req>(
    context: &ExecutionContext<'schema, 'req>,
    fetch: &FetchNode,
    response_lock: &'req RwLock<GraphQLResponse>,
) -> Result<()> {
    if fetch.requires.is_some() {
        panic!("we expect fetch.requires to only defined be on an _entities fetch")
    }

    let service = &context.service_map[&fetch.service_name];

    let mut variables: HashMap<String, Value> = HashMap::new();
    if let Some(vars) = &context.request_context.graphql_request.variables {
        for variable_name in &fetch.variable_usages {
            if let Some(variable) = vars.get(&variable_name) {
                variables.insert(variable_name.to_string(), variable.clone());
            }
        }
    }

    let response_received = service
        .send_operation(context.request_context, fetch.operation.clone(), variables)
        .await?;

    let mut results_to_merge = response_lock.write().await;
    results_to_merge.merge(response_received);

    Ok(())
}

async fn execute_entities_fetch<'schema, 'req>(
    context: &ExecutionContext<'schema, 'req>,
    fetch: &FetchNode,
    response_lock: &'req RwLock<GraphQLResponse>,
) -> Result<()> {
    if fetch.requires.is_none() {
        panic!("_entities fetch without `fetch.requires` ???")
    }

    let service = &context.service_map[&fetch.service_name];

    let requires = letp!(Some(requires) = fetch.requires.as_ref() => requires);

    let mut variables: HashMap<String, Value> = HashMap::new();
    if let Some(vars) = &context.request_context.graphql_request.variables {
        for variable_name in &fetch.variable_usages {
            if let Some(variable) = vars.get(&variable_name) {
                variables.insert(variable_name.to_string(), variable.clone());
            }
        }
    }

    let mut representations_to_entity: Vec<usize> = vec![];

    if variables.contains_key("representations") {
        panic!("variables must not contain key named 'represenations'");
    }

    update_variables_with_representations(
        &mut variables,
        response_lock,
        requires,
        &mut representations_to_entity,
    )
    .await;

    let response_received = service
        .send_operation(context.request_context, fetch.operation.clone(), variables)
        .await?;

    if let Some(recieved_entities) = response_received.data.get("_entities") {
        let mut entities_to_merge = response_lock.write().await;
        entities_to_merge.merge_errors(response_received.errors);
        let data = &mut (*entities_to_merge).data;
        trace!(
            "{{\"merge\": {}, \"into entities\": {}, \"with indexes\": {:?}}}",
            recieved_entities.to_string(),
            data.to_string(),
            representations_to_entity
        );
        match data {
            Value::Array(entities) => {
                for (repr_idx, entity_idx) in representations_to_entity.into_iter().enumerate() {
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
        panic!("Expected data._entities to contain elements");
    }

    Ok(())
}

fn execute_selection_set(entity: &Value, requires: &SelectionSet) -> Value {
    if entity.is_null() {
        return Value::default();
    }

    let mut result: Value = json!({});

    for selection in requires {
        match selection {
            Field(field) => {
                let response_name = field.alias.as_ref().unwrap_or(&field.name);

                if let Some(response_value) = entity.get(response_name) {
                    if let Value::Array(inner) = response_value {
                        result[response_name] = Value::Array(
                            inner
                                .iter()
                                .map(|element| {
                                    if let Some(sub_selections) = &field.selections {
                                        execute_selection_set(element, sub_selections)
                                    } else {
                                        element.clone()
                                    }
                                })
                                .collect(),
                        );
                    } else if let Some(ref selections) = field.selections {
                        result[response_name] = execute_selection_set(response_value, selections);
                    } else {
                        result[response_name] = response_value.clone();
                    }
                } else {
                    panic!(
                        "Field '{}' was not found in response {}",
                        response_name,
                        entity.to_string()
                    );
                }
            }
            InlineFragment(fragment) => {
                // TODO(ran) FIXME: QQQ if there's no type_condition, we don't recurse?
                if let Some(ref type_condition) = fragment.type_condition {
                    if let Some(typename) = entity.get("__typename") {
                        let typename = typename.as_str().expect("__typename's type must be String");
                        if typename == type_condition {
                            merge(
                                &mut result,
                                &execute_selection_set(entity, &fragment.selections),
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
    variables: &mut HashMap<String, Value>,
    response: &RwLock<GraphQLResponse>,
    requires: &SelectionSet,
    representations_to_entity: &mut Vec<usize>,
) {
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
}

// TODO(ran) FIXME: update message on various unreachable!s, convert to Result
// TODO(ran) FIXME: replace panics with Error
