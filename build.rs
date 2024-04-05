#![deny(clippy::unwrap_used)]

use std::collections::HashMap;
use std::error::Error;
use std::path;

use error_stack::{Report, ResultExt};
use prost_build::Config;
use serde::Deserialize;

fn main() -> Result<(), Report<BuildError>> {
    println!("cargo:rerun-if-changed=proto/*.proto");
    println!("cargo:rerun-if-changed=src/proto/*.rs");
    println!("cargo:rerun-if-changed=proto/_attributes.toml");

    let mut proto_files = Vec::new();
    let filewalk = walkdir::WalkDir::new("proto")
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok());

    for file in filewalk {
        if file.metadata().change_context(BuildError)?.is_file() {
            let path = std::fs::canonicalize(file.path()).change_context(BuildError)?;

            let path_name = path
                .to_str()
                .expect("Failed to convert path to string for proto file");
            if let Some(ext) = path.extension() {
                if ext == "proto" {
                    proto_files.push(path_name.to_string());
                }
            }
        }
    }

    let proto_files = proto_files
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<&str>>();
    let mut parents = proto_files
        .iter()
        .map(|s| {
            path::Path::new(s)
                .parent()
                .expect("Failed to get parent of proto file")
                .to_str()
                .expect("Failed to convert parent path to string")
        })
        .collect::<Vec<&str>>();

    parents.dedup();
    let mut protoc = Config::new();
    protoc
        .out_dir("src/proto")
        .format(true)
        .compile_well_known_types()
        .format(true)
        .enable_type_names()
        .include_file("_all.rs");

    load_attributes(&mut protoc).change_context(BuildError)?;

    protoc
        .compile_protos(&proto_files, &parents)
        .change_context(BuildError)?;
    Ok(())
}

fn load_attributes(config: &mut Config) -> Result<(), Report<LoadAttributesError>> {
    let path = path::PathBuf::from("proto/_attributes.json");
    let file = std::fs::File::open(path).change_context(LoadAttributesError)?;

    let attributes =
        serde_json::from_reader::<_, AttributesConfig>(file).change_context(LoadAttributesError)?;

    for (key, value) in attributes.type_attributes {
        for value in value {
            config.type_attribute(key.clone(), value.clone());
        }
    }
    for (key, value) in attributes.field_attributes {
        for value in value {
            config.field_attribute(key.clone(), value.clone());
        }
    }
    for (key, value) in attributes.enum_attributes {
        for value in value {
            config.enum_attribute(key.clone(), value.clone());
        }
    }

    Ok(())
}

#[derive(Debug, Default, Deserialize)]
struct AttributesConfig {
    #[serde(rename = "type", default)]
    type_attributes: HashMap<String, Vec<String>>,
    #[serde(rename = "field", default)]
    field_attributes: HashMap<String, Vec<String>>,
    #[serde(rename = "enum", default)]
    enum_attributes: HashMap<String, Vec<String>>,
}

#[derive(Debug, Default)]
pub struct BuildError;

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "custom build error")
    }
}

impl Error for BuildError {}

#[derive(Debug, Default)]
pub struct LoadAttributesError;

impl std::fmt::Display for LoadAttributesError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "failed to load attributes for proto files")
    }
}

impl Error for LoadAttributesError {}
