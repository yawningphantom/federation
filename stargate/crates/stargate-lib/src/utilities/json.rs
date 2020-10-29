use serde_json::Value;

#[derive(Debug, Clone)]
pub enum JsonSliceValue<'a> {
    Value(&'a serde_json::Value),
    Array(Vec<JsonSliceValue<'a>>),
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
                            .map(|v| JsonSliceValue::from_path_and_value(tail, v))
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
                // TODO(ran) FIXME: This shouldn't panic because we may slice into fields that are
                //  only available on some type conditions. Consider having a more typed approach?
                tracing::debug!(
                    "Attempting to slice with '{}' but there is no key by that name: {}",
                    head,
                    value
                );
                JsonSliceValue::Value(value)
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
        }
    }
}

impl<'a> From<&'a serde_json::Value> for JsonSliceValue<'a> {
    fn from(v: &'a Value) -> Self {
        JsonSliceValue::Value(v)
    }
}
