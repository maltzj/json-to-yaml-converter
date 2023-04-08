extern crate tempdir;

use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Write;
use tempdir::TempDir;

mod conversion;

// First order of operations: write out as one big string
// Next step: create internal yaml graph structure

fn run(args: impl Iterator<Item = String>) -> Result<String, Box<dyn Error>> {
    let collected_args: Vec<String> = args.collect();

    if collected_args.len() != 3 {
        panic!("Args length must be at least three");
    }

    let deserialized_file = deserialize(&collected_args[1])?;
    let file_output = conversion::convert_to_yaml_string(&deserialized_file);
    let writable_file = open_file_for_writing(&collected_args[2])?;

    Ok(String::from("Everything worked"))
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
