use serde_json::Value;

#[derive(Debug)]
pub enum JsonSliceValue<'a> {
    Value(&'a mut serde_json::Value),
    Array(Vec<JsonSliceValue<'a>>),
    Null,
}

// TODO(ran) FIXME: test this.
// TODO(ran) FIXME: change all pub to pub(crate)
impl<'a> JsonSliceValue<'a> {
    pub fn new(v: &'a mut Value) -> Self {
        if let Value::Array(v) = v {
            JsonSliceValue::Array(v.iter_mut().map(JsonSliceValue::new).collect())
        } else {
            JsonSliceValue::Value(v)
        }
    }

    pub fn get_mut(self, key: &String) -> Option<Self> {
        match self {
            JsonSliceValue::Value(v) => v.get_mut(key).map(JsonSliceValue::new),
            JsonSliceValue::Array(_) => None,
            JsonSliceValue::Null => None,
        }
    }

    pub fn slice_by_path(self, path: &[String]) -> Self {
        if path.is_empty() {
            return self;
        }

        if let JsonSliceValue::Null = self {
            return self;
        }

        let (head, tail) = path.split_first().expect("path cannot not be empty");
        if head != "@" {
            match self {
                JsonSliceValue::Array(_) => {
                    unreachable!("when the path component is not @, the type cannot be an array")
                }
                v => v
                    .get_mut(head)
                    .map(|v| v.slice_by_path(tail))
                    .unwrap_or(JsonSliceValue::Null),
            }
        } else {
            match self {
                JsonSliceValue::Array(arr) => {
                    JsonSliceValue::Array(arr.into_iter().map(|v| v.slice_by_path(tail)).collect())
                }
                _ => unreachable!("when the path component is @, the type must be an array"),
            }
        }
    }

    pub fn flatten(self) -> Vec<JsonSliceValue<'a>> {
        match self {
            JsonSliceValue::Array(arr) => arr.into_iter().flat_map(|v| v.flatten()).collect(),
            v => vec![v],
        }
    }
}

// TODO(ran) FIXME: test this.
pub fn slice_and_clone_at_path(value: &Value, path: &[String]) -> Value {
    if path.is_empty() {
        return value.clone();
    }

    let (head, tail) = path.split_first().expect("Verified path is not empty");

    if head == "@" {
        if let Value::Array(array_value) = value {
            Value::Array(
                array_value
                    .iter()
                    .flat_map(|v| match slice_and_clone_at_path(v, tail) {
                        Value::Array(arr) => arr,
                        other => vec![other],
                    })
                    .collect(),
            )
        } else {
            panic!(
                "Attempting to slice with '@' but value is not array: {}",
                value
            )
        }
    } else if let Some(v) = value.get(head) {
        slice_and_clone_at_path(v, tail)
    } else {
        tracing::debug!(
            "Attempting to slice with '{}' but there is no key by that name: {}",
            head,
            value
        );
        Value::Null
    }
}

#[cfg(test)]
mod tests {
    use crate::utilities::json::JsonSliceValue;

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
        let jsvm = JsonSliceValue::new(&mut json);
        let path: Vec<String> = vec!["me", "reviews", "@", "product", "similarBooks", "@"]
            .into_iter()
            .map(String::from)
            .collect();
        // TODO(ran) FIXME: assert equality
        println!("{:?}", jsvm.slice_by_path(&path));
    }
}
