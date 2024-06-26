use bevy::prelude::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

#[derive(Debug, Default, Component)]
pub struct MainMenu;

fn blend_in(mut commands: Commands) {}

fn blend_out(mut commands: Commands, query: Query<Entity, With<MainMenu>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
