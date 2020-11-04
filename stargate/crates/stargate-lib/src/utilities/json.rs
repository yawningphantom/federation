use serde_json::Value;

#[derive(Debug, Clone)]
pub enum JsonSliceValue<'a> {
    Value(&'a serde_json::Value),
    Array(Vec<JsonSliceValue<'a>>),
    Null,
}

// TODO(ran) FIXME: test this.
impl<'a> JsonSliceValue<'a> {
    pub fn from_path_and_value(path: &[String], value: &'a Value) -> JsonSliceValue<'a> {
        if path.is_empty() {
            JsonSliceValue::from(value)
        } else if let Some((head, tail)) = path.split_first() {
            if head == "@" {
                if let Value::Array(array_value) = value {
                    JsonSliceValue::Array(
                        array_value
                            .iter()
                            // .map(|v| JsonSliceValue::from_path_and_value(tail, v))
                            .flat_map(|v| match JsonSliceValue::from_path_and_value(tail, v) {
                                JsonSliceValue::Array(arr) => arr,
                                other => vec![other],
                            })
                            .filter(|v| !matches!(v, JsonSliceValue::Null))
                            .collect(),
                    )
                } else {
                    panic!(
                        "Attempting to slice with '@' but value is not array: {}",
                        value
                    )
                }
            } else if let Some(v) = value.get(head) {
                JsonSliceValue::from_path_and_value(tail, v)
            } else {
                tracing::debug!(
                    "Attempting to slice with '{}' but there is no key by that name: {}",
                    head,
                    value
                );
                JsonSliceValue::Null
            }
        } else {
            unreachable!("verified path is not empty.")
        }
    }

    pub fn into_value(self) -> Value {
        match self {
            JsonSliceValue::Value(v) => v.clone(),
            JsonSliceValue::Array(arr) => {
                Value::Array(arr.into_iter().map(|v| v.into_value()).collect())
            }
            JsonSliceValue::Null => unreachable!("Nulls should have all been filtered"),
            // JsonSliceValue::Null => Value::Null,
        }
    }
}

impl<'a> From<&'a serde_json::Value> for JsonSliceValue<'a> {
    fn from(v: &'a Value) -> Self {
        JsonSliceValue::Value(v)
    }
}
