use bevy::app::App;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};
use bevy::winit::WinitWindows;

#[derive(Debug, Resource)]
pub struct WebUISource {
    pub url: String,
}

/// the web ui plugin borrows code from https://github.com/PawelBis/bevy_wry
///  but has some changes to fit the needs of the game
///  also we use the custom protocol to communicate with the webview
#[derive(Debug, Default)]
pub struct WebUIPlugin;

impl Plugin for WebUIPlugin {
    fn build(&self, app: &mut App) {}
}
