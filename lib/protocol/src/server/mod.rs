use std::time::Duration;

use bevy::app::App;
use bevy::asset::AsyncWriteExt;
use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::local::ServerLocalConnectionAgent;
use crate::proto::play::client::packet::packet_play_client::Payload as ClientPayload;
use crate::proto::play::client::packet::PacketPlayClient;
use crate::proto::play::server::packet::packet_play_server::Payload as ServerPayload;
use crate::proto::play::server::packet::PacketPlayServer;

mod agents;

pub struct ServerProtocolPlugin;

impl Plugin for ServerProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            RepliconPlugins
                .build()
                .disable::<ClientPlugin>()
                .set(ServerPlugin {
                    tick_policy: TickPolicy::EveryFrame,
                    visibility_policy: VisibilityPolicy::All,
                    update_timeout: Duration::from_secs(10),
                })
                .build(),
        );

        app.add_systems(
            Update,
            (
                start_server.run_if(resource_added::<ServerLocalConnectionAgent>),
                stop_server.run_if(resource_removed::<ServerLocalConnectionAgent>()),
            ),
        );
        app.add_systems(
            PreUpdate,
            receive_updates.run_if(resource_exists::<ServerLocalConnectionAgent>),
        );
        app.add_systems(
            PostUpdate,
            send_updates.run_if(resource_exists::<ServerLocalConnectionAgent>),
        );
    }
}

#[derive(Debug, Clone, Component)]
pub struct PlayerProfile {
    pub display_name: String,
}

fn start_server(
    mut replicon_server: ResMut<RepliconServer>,
    mut event_writer: EventWriter<ServerEvent>,
) {
    event_writer.send(ServerEvent::ClientConnected {
        client_id: ClientId::SERVER,
    });
    replicon_server.set_running(true);
}

fn stop_server(mut replicon_server: ResMut<RepliconServer>,
               mut event_writer: EventWriter<ServerEvent>, ) {
    event_writer.send(ServerEvent::ClientDisconnected {
        client_id: ClientId::SERVER,
        reason: "shutdown".to_string(),
    });
    replicon_server.set_running(false);
}

fn send_updates(
    local_connection_agent: Res<ServerLocalConnectionAgent>,
    mut replicon_server: ResMut<RepliconServer>,
) {
    for (client_id, channel, message) in replicon_server.drain_sent() {
        assert_eq!(client_id, ClientId::SERVER);
        let mut message = message.to_vec();
        message.splice(0..0, [channel]);
        local_connection_agent.send(PacketPlayServer {
            payload: Some(ServerPayload::Sync(message)),
        });
    }
}

fn receive_updates(
    local_connection_agent: Res<ServerLocalConnectionAgent>,
    mut replicon_server: ResMut<RepliconServer>,
    mut buffer: Local<Vec<PacketPlayClient>>,
) {
    local_connection_agent.drain(&mut buffer);
    for packet in buffer.drain(..) {
        match packet.payload {
            Some(ClientPayload::Sync(sync_message)) => {
                let channel = sync_message[0];
                let message = sync_message[1..].to_vec();
                replicon_server.insert_received(ClientId::SERVER, channel, message);
            }
            None => continue,
        }
    }
}
