use serde_json::Value;

#[derive(Debug, Clone)]
pub enum JsonSliceValue<'a> {
    Value(&'a serde_json::Value),
    Array(Vec<JsonSliceValue<'a>>),
    Null,
}

#[derive(Debug)]
pub enum JsonSliceValueMut<'a> {
    Value(&'a mut serde_json::Value),
    Array(Vec<JsonSliceValueMut<'a>>),
    Null,
}

// TODO(ran) FIXME: test this.
impl<'a> JsonSliceValueMut<'a> {
    pub fn new(v: &'a mut Value) -> Self {
        if let Value::Array(v) = v {
            JsonSliceValueMut::Array(v.iter_mut().map(JsonSliceValueMut::new).collect())
        } else {
            JsonSliceValueMut::Value(v)
        }
    }

    pub fn get_mut(self, key: &String) -> Option<Self> {
        match self {
            JsonSliceValueMut::Value(v) => v.get_mut(key).map(JsonSliceValueMut::new),
            JsonSliceValueMut::Array(_) => None,
            JsonSliceValueMut::Null => None,
        }
    }

    pub fn slice_by_path(self, path: &[String]) -> Self {
        if path.is_empty() {
            return self;
        }

        if let JsonSliceValueMut::Null = self {
            return self;
        }

        let (head, tail) = path.split_first().expect("path cannot not be empty");
        if head != "@" {
            match self {
                JsonSliceValueMut::Array(_) => {
                    unreachable!("when the path component is not @, the type cannot be an array")
                }
                v => v
                    .get_mut(head)
                    .map(|v| v.slice_by_path(tail))
                    .unwrap_or(JsonSliceValueMut::Null),
            }
        } else {
            match self {
                JsonSliceValueMut::Array(arr) => JsonSliceValueMut::Array(
                    arr.into_iter().map(|v| v.slice_by_path(tail)).collect(),
                ),
                _ => unreachable!("when the path component is @, the type must be an array"),
            }
        }
    }

    pub fn flatten(self) -> Vec<JsonSliceValueMut<'a>> {
        match self {
            JsonSliceValueMut::Array(arr) => arr.into_iter().flat_map(|v| v.flatten()).collect(),
            v => vec![v],
        }
    }
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
                            // .filter(|v| !matches!(v, JsonSliceValue::Null))
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
            // JsonSliceValue::Null => unreachable!("Nulls should have all been filtered"),
            JsonSliceValue::Null => Value::Null,
        }
    }
}

impl<'a> From<&'a serde_json::Value> for JsonSliceValue<'a> {
    fn from(v: &'a Value) -> Self {
        JsonSliceValue::Value(v)
    }
}

#[cfg(test)]
mod tests {
    use crate::utilities::json::JsonSliceValueMut;

    #[test]
    fn test_slice_by_path() {
        let mut json = serde_json::json!({
            "me": {
                "__typename": "User",
                "id": "1",
                "reviews": [
                    {
                        "product": {
                            "__typename": "Furniture"
                        }
                    },
                    {
                        "product": {
                            "__typename": "Furniture"
                        }
                    },
                    {
                        "product": {
                            "__typename": "Book",
                            "isbn": "0201633612",
                            "similarBooks": [
                                {
                                    "__typename": "Book",
                                    "isbn": "0201633612",
                                    "title": "DesignPatterns",
                                    "year": 1995
                                },
                                {
                                    "__typename": "Book",
                                    "isbn": "0136291554",
                                    "title": "ObjectOrientedSoftwareConstruction",
                                    "year": 1997
                                }
                            ]
                        }
                    }
                ]
            }
        });
        let jsvm = JsonSliceValueMut::new(&mut json);
        let path: Vec<String> = vec!["me", "reviews", "@", "product", "similarBooks", "@"]
            .into_iter()
            .map(String::from)
            .collect();
        // TODO(ran) FIXME: assert equality
        println!("{:?}", jsvm.slice_by_path(&path));
    }
}
