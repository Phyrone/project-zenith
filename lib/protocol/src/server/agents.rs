use bevy::prelude::{Entity, Resource};
use bevy_replicon::prelude::ClientId;
use hashbrown::HashMap;

#[derive(Default, Resource)]
pub struct ClientAgentMap {
    agents: HashMap<ClientId, Entity>,
}

impl ClientAgentMap {
    pub fn get(&self, client_id: ClientId) -> Option<Entity> {
        self.agents.get(&client_id).copied()
    }
}
