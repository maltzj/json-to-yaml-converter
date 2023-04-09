use serde_json;
use serde_json::Value;

pub fn convert_to_yaml_string(serde: &Value) -> String {
    // TODO: I really should add a --- directive up top, but that's not strictly necessary.
    let mut result_string = convert_to_yaml_string_internal(&serde, 0).trim().to_string();
    result_string.push_str("\n");
    result_string
}

fn convert_to_yaml_string_internal(serde: &Value, indentation_level: usize) -> String {
    let mut result = String::from("");
    let spaces = " ".repeat(indentation_level);
    match serde {
        Value::Null => (),
        Value::Bool(value) => {
            result.push_str(&format!("{}", value));
        }
        Value::Number(num) => {
            result.push_str(&format!("{}", num));
        }
        Value::String(string) => {
            if string.len() == 0 {
                result.push_str(&format!("''"))
            } else {
                result.push_str(&format!("{}", string));
            }
        }
        Value::Array(vector) => {
            if vector.len() == 0 {
                result.push_str("[]");
            } else {
                result.push_str(&generate_string_for_array(&vector, indentation_level));
            }
        }
        Value::Object(mapping) => {
            if mapping.keys().len() == 0 {
                result.push_str("{}");
            } else {
                let mut i = 0;
                for (key, value) in mapping {
                    let mut is_scalar = false;
                    let mapping_value = match value {
                        Value::Bool(_) | Value::Number(_) | Value::String(_) => {
                            is_scalar = true;
                            convert_to_yaml_string_internal(&value, indentation_level + 2)
                        }
                        Value::Object(internal_object) => {
                            convert_to_yaml_string_internal(&value, indentation_level + 2)
                        }
                        Value::Array(internal_vector) => {
                            convert_to_yaml_string_internal(&value, indentation_level + 2)
                        }
                        _ => "".to_string(),
                    };


                    let newline_to_add = if i == mapping.keys().len() - 1 {
                        ""
                    } else {
                        "\n"
                    };
                    
                    // Scalars are rendered inline vs multi-item arrays and objects, which are
                    // rendered with another level of indentation.  When checking these values, we
                    // trim + re-add our own newline to deal with cases where we would create
                    // multiple newlines (like having a single-element object within a
                    // single-element array.
                    if is_scalar || mapping_value.trim() == "{}" || mapping_value.trim() == "[]" {
                        let spaces_to_use = if i == 0 { "" } else { &spaces };
                        result.push_str(&format!(
                            "{}{}: {}{}",
                            spaces_to_use,
                            key,
                            mapping_value.trim(),
                            newline_to_add
                        ));
                    } else {
                        let extra_spaces = " ".repeat(indentation_level + 2);
                        result.push_str(&format!(
                            "{}:\n{}{}{}",
                            key,
                            extra_spaces,
                            mapping_value.trim(),
                            newline_to_add
                        ));
                    }
                    i += 1
                }
            }
        }
    }
    result.push_str("\n");
    result
}

fn generate_string_for_array(vector: &Vec<Value>, indentation_level: usize) -> String {
    let mut internal_result = String::from("");
    for (index, value) in vector.iter().enumerate() {
        let internal_string = match value {
            Value::Null => "".to_string(),
            Value::Bool(_) | Value::Number(_) | Value::String(_) => {
                convert_to_yaml_string_internal(&value, 0)
            }
            Value::Array(_) => convert_to_yaml_string_internal(&value, indentation_level + 2),
            Value::Object(mapping) => {
                convert_to_yaml_string_internal(&value, indentation_level + 2)
            }
        };
        // Don't indent for index = 0 because we assume that is taken care of by any upper levels.
        // I'm also pretty convinced there's an edge-case in here around multiple newlines getting
        // rendered, but I just haven't found it yet :\.
        if index != 0 {
            let spaces = " ".repeat(indentation_level);
            internal_result.push_str(&format!("{}- {}", spaces, &internal_string));
        } else {
            internal_result.push_str(&format!("- {}", &internal_string));
        }
    }

    internal_result
}

// TODO: Do I want to write some test which round trips these two things and asserts that they're
// equal?  Probably; for now I'll just test this way to get the basics!
#[cfg(test)]
mod parsing_tests {
    use super::*;

    #[test]
    fn it_converts_boolean() {
        let data = "true";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "true\n");
    }

    #[test]
    fn it_converts_numbers() {
        let data = "12";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "12\n");
    }

    #[test]
    fn it_converts_strings() {
        let data = "\"test\"";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "test\n");
    }

    #[test]
    fn it_converts_empty_string() {
        let data = "\"\"";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "''\n");
    }

    #[test]
    fn it_converts_an_array_with_one_element() {
        let data = "[1]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "- 1\n");
    }

    #[test]
    fn it_converts_an_array_with_two_element() {
        let data = "[1, 2]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "- 1\n- 2\n");
    }

    #[test]
    fn it_converts_an_array_with_all_primtive_elements() {
        let data = "[1, false, \"a potato\"]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "- 1\n- false\n- a potato\n");
    }

    #[test]
    fn it_converts_nested_arrays() {
        let data = "[[1]]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "- - 1\n");
    }

    #[test]
    fn it_converts_empty_arrays() {
        let data = "[]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "[]\n");
    }

    #[test]
    fn it_converts_nested_empty_arrays() {
        let data = "[[], [], []]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "- []\n- []\n- []\n");
    }

    #[test]
    fn it_converts_nested_arrays_with_multiple_values() {
        let data = "[[\"a\", 2]]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "- - a\n  - 2\n");
    }

    #[test]
    fn it_handles_multiple_layers_of_nesting() {
        let data = "[[\"a\", [2, 3]]]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "- - a\n  - - 2\n    - 3\n");
    }

    #[test]
    fn it_maps_objects_to_scalars() {
        let data = "{\"a\": 1}";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "a: 1\n");
    }

    #[test]
    fn it_maps_empty_objects() {
        let data = "{}";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "{}\n");
    }

    #[test]
    fn it_maps_objects_to_empty_objects() {
        let data = "{\"a\": {}}";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "a: {}\n");
    }

    #[test]
    fn it_maps_objects_to_nested_objects() {
        let data = "{\"a\": {\"b\": 2}}";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "a:\n  b: 2\n");
    }

    #[test]
    fn it_maps_objects_to_multiple_nested_objects() {
        let data = "{\"a\": {\"b\": 2, \"c\": 3}}";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "a:\n  b: 2\n  c: 3\n");
    }

    #[test]
    fn it_maps_objects_to_arrays() {
        let data = "{\"a\": [\"b\", \"c\"]}";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "a:\n  - b\n  - c\n");
    }

    #[test]
    fn it_maps_nested_objects_and_arrays() {
        let data = "{\"a\": [{\"key\": 1}, \"c\"]}";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "a:\n  - key: 1\n  - c\n");
    }

    #[test]
    fn it_works_with_a_two_element_object_in_an_array() {
        let data = "[{\"a\": 1, \"c\": 2}]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "- a: 1\n  c: 2\n");
    }
}
