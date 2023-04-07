extern crate tempdir;

use serde_json;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Write;
use tempdir::TempDir;

// First order of operations: write out as one big string
// Next step: create internal yaml graph structure

fn run(args: impl Iterator<Item = String>) -> Result<String, Box<dyn Error>> {
    let collected_args: Vec<String> = args.collect();

    if collected_args.len() != 3 {
        panic!("Args length must be at least three");
    }

    let deserialized_file = deserialize(&collected_args[1])?;
    let file_output = convert_to_yaml_string(&deserialized_file);
    let writable_file = open_file_for_writing(&collected_args[2])?;

    Ok(String::from("TODO: Fill me in"))
}

fn deserialize(file_path: &str) -> Result<Value, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let u = serde_json::from_reader(reader)?;

    Ok(u)
}

fn open_file_for_writing(file_path: &str) -> Result<File, Box<dyn Error>> {
    let mut opened_file = File::create(file_path)?;
    Ok(opened_file)
}

fn convert_to_yaml_string(serde: &Value) -> String {
    convert_to_yaml_string_internal(&serde).trim().to_string()
}

fn convert_to_yaml_string_internal(serde: &Value) -> String {
    let mut result = String::from("");
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
                result.push_str("''")
            } else {
                result.push_str(&format!("{}", string));
            }
        }
        Value::Array(vector) => {
            if vector.len() == 0 {
                println!("hitting base case");
                result.push_str("[]");
            } else {
                for value in vector {
                    let internal_string = match value {
                        Value::Bool(_) | Value::Number(_) | Value::String(_) => {
                            convert_to_yaml_string_internal(&value)
                        }
                        Value::Array(ref internal_vector) => {
                            let sub_result = convert_to_yaml_string_internal(&value);
                            let mut final_result = String::from("");
                            for row in sub_result.split("\n") {
                                // Split appears to add an empty element at the end, so ignore
                                // that.
                                if row.len() != 0 {
                                    final_result.push_str(&format!("  {}\n", row));
                                }
                            }

                            final_result.drain(..2);
                            final_result
                        }
                        _ => {
                            panic!("panicking");
                            "".to_string()
                        }
                    };
                    result.push_str(&format!("- {}", &internal_string));
                }
            }
        }
        _ => (),
    }
    println!("Attaching newline");
    result.push_str("\n");
    result
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
        assert_eq!(result, "true");
    }

    #[test]
    fn it_converts_numbers() {
        let data = "12";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "12");
    }

    #[test]
    fn it_converts_strings() {
        let data = "\"test\"";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "test");
    }

    #[test]
    fn it_converts_empty_string() {
        let data = "\"\"";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "''");
    }

    #[test]
    fn it_converts_an_array_with_one_element() {
        let data = "[1]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "- 1");
    }

    #[test]
    fn it_converts_an_array_with_two_element() {
        let data = "[1, 2]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "- 1\n- 2");
    }

    #[test]
    fn it_converts_an_array_with_all_primtive_elements() {
        let data = "[1, false, \"a potato\"]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "- 1\n- false\n- a potato");
    }

    #[test]
    fn it_converts_nested_arrays() {
        let data = "[[1]]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "- - 1");
    }

    #[test]
    fn it_converts_empty_arrays() {
        let data = "[]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "[]");
    }

    #[test]
    fn it_converts_nested_empty_arrays() {
        let data = "[[], [], []]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "- []\n- []\n- []");
    }

    #[test]
    fn it_converts_nested_arrays_with_multiple_values() {
        let data = "[[\"a\", 2]]";
        let result =
            convert_to_yaml_string(&serde_json::from_str(data).expect("Could not parse data"));
        assert_eq!(result, "- - a\n  - 2");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_reads_a_json_file() {
        let temp_dir_path = TempDir::new("directory").expect("couldn't create directory");
        let path = temp_dir_path.path().join("test-file.json");
        let mut temp_file = File::create(&path).expect("failed to create file");
        writeln!(temp_file, "{{\"test\": 2}}").expect("write failed");

        let successfully_deserialized = match path.into_os_string().into_string() {
            Ok(val) => match deserialize(&val) {
                Ok(_) => true,
                Err(error) => {
                    panic!("Failed to deserialize file! {:?}", error);
                }
            },
            Err(error) => panic!("Could not create path! {:?}", error),
        };
        assert!(successfully_deserialized);
    }

    #[test]
    fn it_fails_on_non_json_file() {
        let temp_dir_path = TempDir::new("directory").expect("couldn't create directory");
        let path = temp_dir_path.path().join("test-file.json");
        let mut temp_file = File::create(&path).expect("failed to create file");
        writeln!(temp_file, "{{\\\"key\":}}").expect("write failed");

        let failed_deserialize = match path.into_os_string().into_string() {
            Ok(val) => match deserialize(&val) {
                Ok(_) => panic!("Should not have deserialized file"),
                Err(_) => true,
            },
            Err(error) => panic!("Could not create path! {:?}", error),
        };
        assert!(failed_deserialize);
    }

    #[test]
    fn it_fails_on_non_existent_file() {
        let temp_dir_path = TempDir::new("directory").expect("couldn't create directory");
        let path = temp_dir_path.path().join("test-file.json");
        let mut temp_file = File::create(&path).expect("failed to create file");
        writeln!(temp_file, "{{\"key\":}}").expect("write failed");

        let failed_deserialize = match temp_dir_path
            .path()
            .join("dne")
            .into_os_string()
            .into_string()
        {
            Ok(val) => match deserialize(&val) {
                Ok(_) => panic!("Should not have deserialized file"),
                Err(_) => true,
            },
            Err(error) => panic!("Could not create path! {:?}", error),
        };
        assert!(failed_deserialize);
    }
}
