use serde_json::Value;

pub(crate) fn merge(target: &mut Value, source: &Value) {
    if source.is_null() {
        return;
    }

    match (target, source) {
        (&mut Value::Object(ref mut map), &Value::Object(ref source)) => {
            for (key, source_value) in source {
                let target_value = map.entry(key.as_str()).or_insert_with(|| Value::Null);

                if !target_value.is_null() && (source_value.is_object() || source_value.is_array())
                {
                    merge(target_value, source_value);
                } else {
                    *target_value = source_value.clone();
                }
            }
        }
        (&mut Value::Array(ref mut array), &Value::Array(ref source)) => {
            for (index, source_value) in source.iter().enumerate() {
                if let Some(target_value) = array.get_mut(index) {
                    if !target_value.is_null() && source_value.is_object() {
                        merge(target_value, source_value);
                    } else {
                        *target_value = source_value.clone();
                    }
                } else {
                    array.push(source_value.clone());
                }
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

pub(crate) fn merge2(target: &mut Value, source: Value) {
    if source.is_null() {
        return;
    }

    // TODO(ran) FIXME: warn on mismatching types
    match target {
        Value::Object(ref mut map) if source.is_object() => {
            let source = letp!(Value::Object(source) = source => source);
            for (key, source_value) in source.into_iter() {
                let target_value = map.entry(key.as_str()).or_insert_with(|| Value::Null);
                if !target_value.is_null() && (source_value.is_object() || source_value.is_array())
                {
                    merge2(target_value, source_value);
                } else {
                    *target_value = source_value;
                }
            }
        }
        Value::Array(ref mut array) if source.is_array() => {
            let mut source = letp!(Value::Array(source) = source => source);
            match (array.len(), source.len()) {
                (x, y) if x == y => {
                    for (src, target) in source.into_iter().zip(array) {
                        merge2(target, src)
                    }
                }
                (x, y) if x < y => {
                    let rest = source.split_off(array.len());
                    for (src, target) in source.into_iter().zip(array.iter_mut()) {
                        merge2(target, src)
                    }
                    array.extend(rest)
                }
                (x, y) if x > y => {
                    for (index, item) in source.into_iter().enumerate() {
                        merge2(array.get_mut(index).unwrap(), item)
                    }
                }
                _ => unreachable!(),
            }
        }
        _ => *target = source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // TODO(ran) FIXME: add test cases to address missing coverage lines (run grcov.sh)

    #[test]
    fn it_should_merge_objects() {
        let mut first: Value = json!({"value1":"a","value2":"b"});
        let second: Value = json!({"value1":"a","value2":"c","value3":"d"});

        merge(&mut first, &second);

        assert_eq!(
            r#"{"value1":"a","value2":"c","value3":"d"}"#,
            first.to_string()
        );
    }
    #[test]
    fn it_should_merge_objects_in_arrays() {
        let mut first: Value = json!([{"value":"a","value2":"a+"},{"value":"b"}]);
        let second: Value = json!([{"value":"b"},{"value":"c"}]);

        merge(&mut first, &second);
        assert_eq!(
            r#"[{"value":"b","value2":"a+"},{"value":"c"}]"#,
            first.to_string()
        );
    }
    #[test]
    fn it_should_merge_nested_objects() {
        let mut first: Value = json!({"a":1,"b":{"someProperty":1,"overwrittenProperty":"clean"}});
        let second: Value = json!({"b":{"overwrittenProperty":"dirty","newProperty":"new"},"c":4});

        merge(&mut first, &second);

        assert_eq!(
            json!({"a":1,"b":{"someProperty":1,"overwrittenProperty":"dirty","newProperty":"new"},"c":4}),
            first
        );
    }
    #[test]
    fn it_should_merge_nested_objects_in_arrays() {
        let mut first: Value = json!({"a":1,"b":[{"c":1,"d":2}]});

        let second: Value = json!({"e":2,"b":[{"f":3}]});

        merge(&mut first, &second);

        assert_eq!(
            r#"{"a":1,"b":[{"c":1,"d":2,"f":3}],"e":2}"#,
            first.to_string()
        );
    }
}
