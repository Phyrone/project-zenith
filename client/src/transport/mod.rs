use bevy::prelude::*;
use bevy_replicon::prelude::*;

pub struct TransportPlugin;

impl Plugin for TransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            RepliconPlugins
                .build()
                .disable::<ServerPlugin>()
                .enable::<ClientPlugin>()
                .build(),
        );
        app.add_systems(Startup, init_client);
        app.add_systems(PostUpdate, client_send_drain.after(ClientSet::Send));
    }
}

fn init_client(mut client: ResMut<RepliconClient>) {
    client.set_status(RepliconClientStatus::Connected {
        client_id: Some(ClientId::new(1)),
    });
}

fn client_send_drain(channels: Res<RepliconChannels>, mut client: ResMut<RepliconClient>) {
    let client_channels = channels.client_channels();
    for (channel_id, message) in client.drain_sent() {
        let channel = &client_channels[channel_id as usize];
        let msg = hex::encode(&message);

        println!(" >-{channel_id:>3}-> {msg} ({:?})", channel.kind);
    }
}
