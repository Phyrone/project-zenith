use std::sync::RwLock;

struct AgentSpawner {}

pub struct PlayerAgent {
    ingress_buffer: RwLock<Vec<u8>>,
}
