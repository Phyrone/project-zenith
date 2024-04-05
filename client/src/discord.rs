use std::ops::{Deref, DerefMut};

use bevy::app::{App, Startup};
use bevy::prelude::{ResMut, Resource};
use discord_rpc_client::Client as DRPCClient;

#[derive(Debug, Default)]
pub struct DiscordRPCPlugin;

impl bevy::prelude::Plugin for DiscordRPCPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DiscordRPCResource {
            client: DRPCClient::new(1122821350340444313),
        })
        .add_systems(Startup, start_client);
    }
}

#[derive(Resource)]
struct DiscordRPCResource {
    client: DRPCClient,
}

impl Deref for DiscordRPCResource {
    type Target = DRPCClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for DiscordRPCResource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

fn start_client(mut discord: ResMut<DiscordRPCResource>) {
    discord.start();
    discord
        .set_activity(|activity| activity.state("Coming Soon"))
        .expect("failed to set activity");
}
