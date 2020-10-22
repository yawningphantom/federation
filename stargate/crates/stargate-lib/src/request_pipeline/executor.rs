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
use tracing::instrument;

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
                let _fetch_result = execute_fetch(context, &fetch_node, response_lock).await;
                //   if fetch_result.is_err() {
                //       context.errors.push(fetch_result.errors)
                //   }
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
                    merge_flattend_results(
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

fn merge_flattend_results(
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

    fn merge_flattend_data(parent_data: &mut Value, child_data: Value, path: &[String]) {
        if path.is_empty() || child_data.is_null() {
            merge(&mut *parent_data, &child_data);
            return;
        }

        if let Some((path_head, path_tail)) = path.split_first() {
            if path_head == "@" {
                if parent_data.is_array() && child_data.is_array() {
                    let child_array = letp!(Value::Array(a) = child_data => a);
                    assert_eq!(
                        parent_data.as_array().unwrap().len(),
                        child_array.len(),
                        "parent and child are not the same length"
                    );

                    parent_data
                        .as_array_mut()
                        .unwrap()
                        .iter_mut()
                        .zip(child_array.into_iter())
                        .for_each(|(parent_entity, child_entity)| {
                            merge_flattend_data(parent_entity, child_entity, path_tail)
                        });
                } else {
                    unreachable!(
                        "merge_flattend_data trying to merge mismatching values, path has `@` but parent/child is not array."
                    )
                }
            } else if parent_data.get(&path_head).is_some() {
                let inner = parent_data.get_mut(&path_head).unwrap();
                merge_flattend_data(inner, child_data, path_tail);
            } else {
                unreachable!("merge_flattend_data trying to merge mismatching values")
            }
        }
    }

    merge_flattend_data(&mut parent_response.data, child_response.data, path)
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

    if let Some(requires) = &fetch.requires {
        let mut representations: Vec<Value> = vec![];
        if variables.contains_key("representations") {
            unimplemented!(
                "Need to throw here because `Variables cannot contain key 'represenations'"
            );
        }

        let results = response_lock.read().await;

        // TODO(ran) FIXME: QQQ: What's the process here? how could results be an array?
        //  What is the existence of __typename tell us?
        let representation_variables = match &results.data {
            Value::Array(entities) => {
                for (index, entity) in entities.iter().enumerate() {
                    let representation = execute_selection_set(entity, requires);
                    if representation.is_object() && representation.get("__typename").is_some() {
                        representations.push(representation);
                        representations_to_entity.push(index);
                    }
                }
                Value::Array(representations)
            }
            Value::Object(_) => {
                let representation = execute_selection_set(&results.data, requires);
                if representation.is_object() && representation.get("__typename").is_some() {
                    representations.push(representation);
                    representations_to_entity.push(0);
                }
                Value::Array(representations)
            }
            _ => {
                println!("In empty match line 199");
                Value::Array(vec![])
            }
        };

        variables.insert("representations".to_string(), representation_variables);
    }

    let data_received = service
        .send_operation(context.request_context, fetch.operation.clone(), variables)
        .await?;

    if let Some(_requires) = &fetch.requires {
        if let Some(recieved_entities) = data_received.get("_entities") {
            let mut entities_to_merge = response_lock.write().await;
            match &(*entities_to_merge).data {
                Value::Array(_entities) => {
                    let entities = entities_to_merge.data.as_array_mut().unwrap();
                    for index in 0..entities.len() {
                        if let Some(rep_index) = representations_to_entity.get(index) {
                            let result = entities.get_mut(*rep_index).unwrap();
                            merge(result, &recieved_entities[index]);
                        }
                    }
                }
                Value::Object(_entity) => {
                    merge(&mut (*entities_to_merge).data, &recieved_entities[0]);
                }
                _ => {}
            }
        } else {
            unimplemented!("Expexected data._entities to contain elements");
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
                    unimplemented!("Field was not found in response");
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
