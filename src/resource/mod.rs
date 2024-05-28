use std::any::Any;

use unstructured::Document;

pub const NAMESPACE_CORE: &str = "core";

//TODO document
/// A resource key represents an identifier for a resource in its scope.
#[derive(
    Debug,
    Eq,
    PartialEq,
    Clone,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    deepsize::DeepSizeOf,
)]
pub struct ResourceKey {
    #[serde(rename = "n")]
    pub namespace: String,
    #[serde(rename = "p")]
    pub name: String,
}

impl ResourceKey {
    fn new(namespace: &str, name: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            name: name.to_string(),
        }
    }
}

#[derive(
    Debug, Eq, PartialEq, Clone, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct BlockData {
    #[serde(rename = "m", alias = "material")]
    pub material: ResourceKey,
    #[serde(rename = "d", alias = "data")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Box<Document>>,
}

pub trait Resource: Any {
    fn key(&self) -> ResourceKey;
    fn get_data(&self) -> Option<Document>;
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::humanize::humanize_memory;

    use super::*;

    #[test]
    pub fn test_msgpack() {
        let id = BlockData {
            material: ResourceKey {
                namespace: NAMESPACE_CORE.to_string(),
                name: "dirt".to_string(),
            },
            data: None,
        };
        let bytes = rmp_serde::to_vec(&id).unwrap();
        let hex = hex::encode(&bytes);

        println!("ver1: {}", hex);
        let json = json!({
            "n": "core",
            "p": "dirt"
        });

        let bytes = rmp_serde::to_vec(&json).unwrap();
        let hex = hex::encode(&bytes);
        println!("ver2: {}", hex);
    }

    #[test]
    pub fn test_de() {
        let expected = ResourceKey {
            namespace: "core".to_string(),
            name: "dirt".to_string(),
        };

        let hex = "92a4636f7265a464697274";
        let bytes = hex::decode(hex).unwrap();
        let data_1 = rmp_serde::from_slice::<ResourceKey>(&bytes).unwrap();
        println!("ver1: {:#?}", data_1);
        let hex = "82a16ea4636f7265a170a464697274";
        let bytes = hex::decode(hex).unwrap();
        let data_2 = rmp_serde::from_slice::<ResourceKey>(&bytes).unwrap();
        println!("ver2: {:#?}", data_2);
        assert_eq!(data_1, data_2);

        assert_eq!(data_1, expected);
        assert_eq!(data_2, expected);
    }

    #[test]
    pub fn sizes() {
        println!(
            "ResourceKey: {}",
            humanize_memory(std::mem::size_of::<ResourceKey>())
        );
        println!(
            "BlockData: {}",
            humanize_memory(std::mem::size_of::<BlockData>())
        );
    }
    
    #[test]
    pub fn test_kv(){

        
    }
}
