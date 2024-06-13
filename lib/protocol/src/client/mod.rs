use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::local::ClientLocalConnectionAgent;
use crate::proto::play::client::packet::packet_play_client::Payload as ClientPayload;
use crate::proto::play::client::packet::PacketPlayClient;
use crate::proto::play::server::packet::packet_play_server::Payload as ServerPayload;
use crate::proto::play::server::packet::PacketPlayServer;

pub struct ClientProtocolPlugin;

impl Plugin for ClientProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RepliconPlugins.build().disable::<ServerPlugin>().build());
        app.add_systems(
            PreUpdate,
            (
                local_server_running.run_if(resource_added::<ClientLocalConnectionAgent>),
                local_server_stopped.run_if(resource_removed::<ClientLocalConnectionAgent>()),
            ),
        );
        app.add_systems(
            PreUpdate,
            receive_updates.run_if(resource_exists::<ClientLocalConnectionAgent>)
                           .after(local_server_running)
                           .after(local_server_stopped),
        );
        app.add_systems(
            PostUpdate,
            send_updates.run_if(resource_exists::<ClientLocalConnectionAgent>),
        );
    }
}

fn local_server_running(
    mut replicon_client: ResMut<RepliconClient>,
) {
    replicon_client.set_status(RepliconClientStatus::Connected {
        client_id: Some(ClientId::SERVER),
    });
    
}

fn local_server_stopped(mut replicon_client: ResMut<RepliconClient>) {
    replicon_client.set_status(RepliconClientStatus::Disconnected);
}

fn send_updates(
    local_connection_agent: Res<ClientLocalConnectionAgent>,
    mut replicon_client: ResMut<RepliconClient>,
) {
    for (channel, message) in replicon_client.drain_sent() {
        let mut sync_message = message.to_vec();
        sync_message.splice(0..0, [channel]);
        local_connection_agent.send(PacketPlayClient {
            payload: Some(ClientPayload::Sync(sync_message)),
        });
    }
}

fn receive_updates(
    local_connection_agent: Res<ClientLocalConnectionAgent>,
    mut replicon_client: ResMut<RepliconClient>,
    mut buffer: Local<Vec<PacketPlayServer>>,
) {
    local_connection_agent.drain(&mut buffer);
    for packet in buffer.drain(..) {
        match packet.payload {
            Some(ServerPayload::Sync(sync_message)) => {
                let channel = sync_message[0];
                let message = sync_message[1..].to_vec();
                replicon_client.insert_received(channel, message);
            }
            None => continue,
        }
    }
}
