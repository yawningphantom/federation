use crate::utilities::deep_merge::deep_merge;
use crate::Stargate;
use http::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLRequest {
    pub query: String,
    pub operation_name: Option<String>,
    pub variables: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GraphQLResponse {
    #[serde(skip_serializing_if = "value_is_null", default)]
    pub data: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<GraphQLError>>,
}

impl GraphQLResponse {
    pub fn merge(&mut self, other: Self) {
        self.merge_data(other.data);
        self.merge_errors(other.errors);
    }

    pub fn merge_data(&mut self, data: Value) {
        deep_merge(&mut self.data, data)
    }

    pub fn merge_errors(&mut self, errors: Option<Vec<GraphQLError>>) {
        if let Some(errors) = errors {
            match self.errors {
                None => self.errors = Some(errors),
                Some(ref mut self_errors) => self_errors.extend(errors.into_iter()),
            }
        }
    }
}

impl Default for GraphQLResponse {
    fn default() -> Self {
        Self {
            data: Value::default(),
            errors: None,
        }
    }
}

fn value_is_null(value: &Value) -> bool {
    matches!(value, Value::Null)
}

/// Extensions to the error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct ErrorExtensionValues(BTreeMap<String, Value>);

/// An error with a message and optional extensions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphQLError {
    /// The error message.
    pub message: String,
    /// Extensions to the error.
    #[serde(skip_serializing_if = "error_extensions_is_empty")]
    pub extensions: Option<ErrorExtensionValues>,
}

fn error_extensions_is_empty(values: &Option<ErrorExtensionValues>) -> bool {
    match values {
        Some(values) => values.0.is_empty(),
        None => true,
    }
}

pub struct RequestContext<'req> {
    pub graphql_request: GraphQLRequest,
    pub header_map: HeaderMap<&'req HeaderValue>,
}

#[derive(Debug)]
pub struct ServerState<'app> {
    pub stargate: Stargate<'app>,
}

#[cfg(test)]
mod tests {
    use crate::transports::http::GraphQLResponse;
    use serde_json::json;

    #[test]
    fn test_skip_serializing() {
        assert_eq!(
            serde_json::to_string(&GraphQLResponse::default()).unwrap(),
            "{}"
        )
    }

    #[test]
    fn test_response_merge_data() {
        let mut r = GraphQLResponse::default();
        r.merge_data(json!({"hello": "world"}));
        assert_eq!(r.data, json!({"hello": "world"}));
        r.merge_data(json!({"hello": "world2"}));
        assert_eq!(r.data, json!({"hello": "world2"}));
        r.merge_data(json!({"Jerry": "Hello"}));
        assert_eq!(r.data, json!({"hello": "world2", "Jerry": "Hello"}));
        r.merge_data(json!({"Jerry": {"Hello": "Uncle Leo!"}}));
        assert_eq!(
            r.data,
            json!({"hello": "world2", "Jerry": {"Hello": "Uncle Leo!"}})
        );
    }

    #[test]
    fn test_default_deserialization() {
        assert_eq!(
            serde_json::from_str::<GraphQLResponse>("{}").unwrap(),
            GraphQLResponse {
                data: serde_json::Value::Null,
                errors: None
            }
        )
    }
}
