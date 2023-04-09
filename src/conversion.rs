use serde_json;
use serde_json::Value;
use std::collections::HashMap;

enum Tag {
    Null,
    Boolean,
    String,
    Integer,
    Float,
    Sequence,
    Mapping,
    Custom(String),
}

enum NodeType {
    SequenceNode(Vec<YAMLNode>),
    MappingNode(HashMap<String, YAMLNode>),
    ScalarNode(String),
}

struct YAMLNode {
    tag: Tag,
    node_type: NodeType,
}

pub fn convert_to_yaml_string(serde: &Value) -> String {
    let mut result_string = convert_to_yaml_string_internal(&serde, 0)
        .trim()
        .to_string();
    result_string.insert_str(0, "---\n");
    result_string.push_str("\n");
    result_string
}

fn convert_to_internal_yaml_representation(serde: &Value) -> YAMLNode {
    match serde {
        Value::Null => {
            YAMLNode {
                tag: Tag::Null,
                node_type: NodeType::ScalarNode("".to_string()),
            }
        }
        Value::Bool(value) => {
            YAMLNode {
                tag: Tag::Boolean,
                node_type: NodeType::ScalarNode(format!("{}", value)),
            }
        }
        Value::Number(number) => {

            YAMLNode {
                tag: if number.is_f64() { Tag::Float} else {Tag::Integer}, // TODO: handle floats and ints.
                node_type: NodeType::ScalarNode(format!("{}", number)),
            }
        }
        Value::String(string) => {
            YAMLNode {
                tag: Tag::String,
                node_type: NodeType::ScalarNode(string.clone()),
            }
        }
        Value::Array(elements) => {
            let mut elements_vector = Vec::new();
            
            for element in elements {
               elements_vector.push(convert_to_internal_yaml_representation(element));  
            }
            YAMLNode {
                tag: Tag::Sequence,
                node_type: NodeType::SequenceNode(elements_vector),
            }
        }
        Value::Object(mapping) => {
            let mut elements_mapping = HashMap::new();

            // TODO: does either standard have an opinion about duplicate keys?
            for (key, value) in mapping {
                elements_mapping.insert(key.clone(), convert_to_internal_yaml_representation(value));
            }

            YAMLNode {
                tag: Tag::Mapping,
                node_type: NodeType::MappingNode(elements_mapping),
            }
        }
    }
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
                        Value::Object(_) => {
                            convert_to_yaml_string_internal(&value, indentation_level + 2)
                        }
                        Value::Array(_) => {
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
            Value::Object(_) => {
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
        assert_contents_match(&result, "true");
    }

    #[test]
    fn it_converts_numbers() {
        let data = "12";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "12");
    }

    #[test]
    fn it_converts_strings() {
        let data = "\"test\"";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "test");
    }

    #[test]
    fn it_converts_empty_string() {
        let data = "\"\"";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "''");
    }

    #[test]
    fn it_converts_an_array_with_one_element() {
        let data = "[1]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "- 1");
    }

    #[test]
    fn it_converts_an_array_with_two_element() {
        let data = "[1, 2]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "- 1\n- 2");
    }

    #[test]
    fn it_converts_an_array_with_all_primtive_elements() {
        let data = "[1, false, \"a potato\"]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "- 1\n- false\n- a potato");
    }

    #[test]
    fn it_converts_nested_arrays() {
        let data = "[[1]]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "- - 1");
    }

    #[test]
    fn it_converts_empty_arrays() {
        let data = "[]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "[]");
    }

    #[test]
    fn it_converts_nested_empty_arrays() {
        let data = "[[], [], []]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "- []\n- []\n- []");
    }

    #[test]
    fn it_converts_nested_arrays_with_multiple_values() {
        let data = "[[\"a\", 2]]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "- - a\n  - 2");
    }

    #[test]
    fn it_handles_multiple_layers_of_nesting() {
        let data = "[[\"a\", [2, 3]]]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "- - a\n  - - 2\n    - 3");
    }

    #[test]
    fn it_maps_objects_to_scalars() {
        let data = "{\"a\": 1}";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "a: 1");
    }

    #[test]
    fn it_maps_empty_objects() {
        let data = "{}";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "{}");
    }

    #[test]
    fn it_maps_objects_to_empty_objects() {
        let data = "{\"a\": {}}";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "a: {}");
    }

    #[test]
    fn it_maps_objects_to_nested_objects() {
        let data = "{\"a\": {\"b\": 2}}";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "a:\n  b: 2");
    }

    #[test]
    fn it_maps_objects_to_multiple_nested_objects() {
        let data = "{\"a\": {\"b\": 2, \"c\": 3}}";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "a:\n  b: 2\n  c: 3");
    }

    #[test]
    fn it_maps_objects_to_arrays() {
        let data = "{\"a\": [\"b\", \"c\"]}";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "a:\n  - b\n  - c");
    }

    #[test]
    fn it_maps_nested_objects_and_arrays() {
        let data = "{\"a\": [{\"key\": 1}, \"c\"]}";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "a:\n  - key: 1\n  - c");
    }

    #[test]
    fn it_works_with_a_two_element_object_in_an_array() {
        let data = "[{\"a\": 1, \"c\": 2}]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_contents_match(&result, "- a: 1\n  c: 2");
    }

    fn assert_contents_match(actual: &str, expected: &str) -> () {
        let mut result_with_prefix_and_suffix = expected.trim().to_string();
        result_with_prefix_and_suffix.push_str("\n");
        result_with_prefix_and_suffix.insert_str(0, "---\n");
        assert_eq!(actual, result_with_prefix_and_suffix);
    }
}
