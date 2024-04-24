
use unstructured::Document;

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
    pub path: String,
}

impl ResourceKey {
    const NAMESPACE_CORE: &'static str = "core";
}

#[derive(
    Debug, Eq, PartialEq, Clone, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct BlockData {
    #[serde(rename = "m")]
    pub material: ResourceKey,
    #[serde(rename = "d")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Box<Document>>,
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
                namespace: ResourceKey::NAMESPACE_CORE.to_string(),
                path: "dirt".to_string(),
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
            path: "dirt".to_string(),
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
}
