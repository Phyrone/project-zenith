use quoted_string::spec::{PartialCodePoint, WithoutQuotingValidator};
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
rkyv::Archive,
rkyv::Serialize,
rkyv::Deserialize,
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

struct NoSlashValidator;
impl WithoutQuotingValidator for NoSlashValidator {
    fn next(&mut self, pcp: PartialCodePoint) -> bool {
        pcp.as_u8() != b'/';
        
        todo!()
    }

    fn end(&self) -> bool {
        todo!()
    }
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
    pub fn test_rkyv() {
        let id = ResourceKey {
            namespace: ResourceKey::NAMESPACE_CORE.to_string(),
            path: "dirt".to_string(),
        };

        let bytes = rkyv::to_bytes::<_, 100_usize>(&id).unwrap();
        let hex = hex::encode(&bytes);

        println!("{}", hex);
    }

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
        let hex = "92a4636f7265a464697274";
        let bytes = hex::decode(hex).unwrap();
        let data = rmp_serde::from_slice::<ResourceKey>(&bytes).unwrap();
        println!("ver1: {:#?}", data);
        let hex = "82a16ea4636f7265a170a464697274";
        let bytes = hex::decode(hex).unwrap();
        let data = rmp_serde::from_slice::<ResourceKey>(&bytes).unwrap();
        println!("ver2: {:#?}", data);
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
