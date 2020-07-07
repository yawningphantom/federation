extern crate wasm_bindgen;

use apollo_query_planner::model::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = getQueryPlan)]
pub fn get_query_plan() -> JsValue {
    JsValue::from_serde(&query_plan()).unwrap()
}

fn query_plan() -> QueryPlan {
    QueryPlan {
        node: Some(PlanNode::Sequence {
            nodes: vec![
                PlanNode::Fetch(FetchNode {
                    service_name: "product".to_owned(),
                    variable_usages: vec![],
                    requires: None,
                    operation: "{topProducts{__typename ...on Book{__typename isbn}...on Furniture{name}}product(upc:\"1\"){__typename ...on Book{__typename isbn}...on Furniture{name}}}".to_owned(),
                }),
                PlanNode::Parallel {
                    nodes: vec![
                        PlanNode::Sequence {
                            nodes: vec![
                                PlanNode::Flatten(FlattenNode {
                                    path: vec![
                                        ResponsePathElement::Field("topProducts".to_owned()), ResponsePathElement::Field("@".to_owned())],
                                    node: Box::new(PlanNode::Fetch(FetchNode {
                                        service_name: "books".to_owned(),
                                        variable_usages: vec![],
                                        requires: Some(vec![
                                            Selection::InlineFragment(InlineFragment {
                                                type_condition: Some("Book".to_owned()),
                                                selections: vec![
                                                    Selection::Field(Field {
                                                        alias: None,
                                                        name: "__typename".to_owned(),
                                                        selections: None,
                                                    }),
                                                    Selection::Field(Field {
                                                        alias: None,
                                                        name: "isbn".to_owned(),
                                                        selections: None,
                                                    })],
                                            })]),
                                        operation: "query($representations:[_Any!]!){_entities(representations:$representations){...on Book{__typename isbn title year}}}".to_owned(),
                                    })),
                                }),
                                PlanNode::Flatten(FlattenNode {
                                    path: vec![
                                        ResponsePathElement::Field("topProducts".to_owned()),
                                        ResponsePathElement::Field("@".to_owned())],
                                    node: Box::new(PlanNode::Fetch(FetchNode {
                                        service_name: "product".to_owned(),
                                        variable_usages: vec![],
                                        requires: Some(vec![
                                            Selection::InlineFragment(InlineFragment {
                                                type_condition: Some("Book".to_owned()),
                                                selections: vec![
                                                    Selection::Field(Field {
                                                        alias: None,
                                                        name: "__typename".to_owned(),
                                                        selections: None,
                                                    }),
                                                    Selection::Field(Field {
                                                        alias: None,
                                                        name: "isbn".to_owned(),
                                                        selections: None,
                                                    }),
                                                    Selection::Field(Field {
                                                        alias: None,
                                                        name: "title".to_owned(),
                                                        selections: None,
                                                    }),
                                                    Selection::Field(Field {
                                                        alias: None,
                                                        name: "year".to_owned(),
                                                        selections: None,
                                                    })],
                                            })]),
                                        operation: "query($representations:[_Any!]!){_entities(representations:$representations){...on Book{name}}}".to_owned(),
                                    })),
                                })]
                        },
                        PlanNode::Sequence {
                            nodes: vec![
                                PlanNode::Flatten(FlattenNode {
                                    path: vec![
                                        ResponsePathElement::Field("product".to_owned())],
                                    node: Box::new(PlanNode::Fetch(FetchNode {
                                        service_name: "books".to_owned(),
                                        variable_usages: vec![],
                                        requires: Some(vec![
                                            Selection::InlineFragment(InlineFragment {
                                                type_condition: Some("Book".to_owned()),
                                                selections: vec![
                                                    Selection::Field(Field {
                                                        alias: None,
                                                        name: "__typename".to_owned(),
                                                        selections: None,
                                                    }),
                                                    Selection::Field(Field {
                                                        alias: None,
                                                        name: "isbn".to_owned(),
                                                        selections: None,
                                                    })],
                                            })]),
                                        operation: "query($representations:[_Any!]!){_entities(representations:$representations){...on Book{__typename isbn title year}}}".to_owned(),
                                    })),
                                }),
                                PlanNode::Flatten(FlattenNode {
                                    path: vec![
                                        ResponsePathElement::Field("product".to_owned())],
                                    node: Box::new(PlanNode::Fetch(FetchNode {
                                        service_name: "product".to_owned(),
                                        variable_usages: vec![],
                                        requires: Some(vec![
                                            Selection::InlineFragment(InlineFragment {
                                                type_condition: Some("Book".to_owned()),
                                                selections: vec![
                                                    Selection::Field(Field {
                                                        alias: None,
                                                        name: "__typename".to_owned(),
                                                        selections: None,
                                                    }),
                                                    Selection::Field(Field {
                                                        alias: None,
                                                        name: "isbn".to_owned(),
                                                        selections: None,
                                                    }),
                                                    Selection::Field(Field {
                                                        alias: None,
                                                        name: "title".to_owned(),
                                                        selections: None,
                                                    }),
                                                    Selection::Field(Field {
                                                        alias: None,
                                                        name: "year".to_owned(),
                                                        selections: None,
                                                    })],
                                            })]),
                                        operation: "query($representations:[_Any!]!){_entities(representations:$representations){...on Book{name}}}".to_owned(),
                                    })),
                                })]
                        }]
                }]
        })
    }
}
