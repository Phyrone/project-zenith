use bevy::app::App;
use bevy::prelude::*;

mod main_menu;

#[derive(Debug)]
pub enum ActiveUi {
    None,
    MainMenu,
    Settings,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {}
}
