use serde_json::Value;

// TODO(ran) FIXME: move to json
pub(crate) fn deep_merge(target: &mut Value, source: Value) {
    if source.is_null() {
        return;
    }

    if target == &source {
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
                    deep_merge(target_value, source_value);
                } else {
                    *target_value = source_value;
                }
            }
        }
        Value::Array(ref mut target) if source.is_array() => {
            let mut source = letp!(Value::Array(source) = source => source);
            let source_len = source.len();
            let target_len = target.len();
            if source_len == target_len {
                for (src, target) in source.into_iter().zip(target) {
                    deep_merge(target, src)
                }
            } else if source_len > target_len {
                let rest = source.split_off(target.len());
                for (src, target) in source.into_iter().zip(target.iter_mut()) {
                    deep_merge(target, src)
                }
                target.extend(rest)
            } else {
                for (target, src) in target.iter_mut().zip(source) {
                    deep_merge(target, src)
                }
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

        deep_merge(&mut first, second);

        assert_eq!(
            r#"{"value1":"a","value2":"c","value3":"d"}"#,
            first.to_string()
        );
    }
    #[test]
    fn it_should_merge_objects_in_arrays() {
        let mut first: Value = json!([{"value":"a","value2":"a+"},{"value":"b"}]);
        let second: Value = json!([{"value":"b"},{"value":"c"}]);

        deep_merge(&mut first, second);
        assert_eq!(
            r#"[{"value":"b","value2":"a+"},{"value":"c"}]"#,
            first.to_string()
        );
    }
    #[test]
    fn it_should_merge_nested_objects() {
        let mut first: Value = json!({"a":1,"b":{"someProperty":1,"overwrittenProperty":"clean"}});
        let second: Value = json!({"b":{"overwrittenProperty":"dirty","newProperty":"new"},"c":4});

        deep_merge(&mut first, second);

        assert_eq!(
            json!({"a":1,"b":{"someProperty":1,"overwrittenProperty":"dirty","newProperty":"new"},"c":4}),
            first
        );
    }
    #[test]
    fn it_should_merge_nested_objects_in_arrays() {
        let mut first: Value = json!({"a":1,"b":[{"c":1,"d":2}]});

        let second: Value = json!({"e":2,"b":[{"f":3}]});

        deep_merge(&mut first, second);

        assert_eq!(
            r#"{"a":1,"b":[{"c":1,"d":2,"f":3}],"e":2}"#,
            first.to_string()
        );
    }
}
