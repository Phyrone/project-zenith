use bevy::app::App;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};
use bevy::winit::WinitWindows;
use http::header::CONTENT_TYPE;
use wry::http::Request;
use wry::{http, WebView, WebViewBuilder};

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
    fn build(&self, app: &mut App) {
        app.insert_resource(WebUISource {
            url: "core://index.html".to_string(),
        })
        .add_systems(
            Startup,
            (setup_wry_system, initial_keep_webview_fullscreen).chain(),
        )
        .add_systems(Update, (keep_webview_fullscreen, handle_source_change));
    }
}

/// this setup creates a wry webview on top
///   the webview is used for everything ui related
/// why wry? because web is easy and the fastest way to make fancy ui
/// do some people want to ki** me for that? probably
/// btw. it also makes modding easier
///
fn setup_wry_system(world: &mut World) {
    let primary_window_entity = world
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(world);

    //TODO better error handling
    let primary_window = world
        .get_non_send_resource::<WinitWindows>()
        .expect("no winit windows found")
        .get_window(primary_window_entity)
        .expect("no primary window found");

    let webview = WebViewBuilder::new_as_child(primary_window)
        .with_transparent(true)
        .with_clipboard(false)
        .with_visible(true)
        .with_accept_first_mouse(false)
        .with_bounds(wry::Rect {
            x: 0,
            y: 0,
            width: 1000,
            height: 1000,
        })
        //first difference is we use custom protocol to get resources
        .with_custom_protocol("core".to_string(), |request| {
            match get_core_ui_response(request) {
                Ok(r) => r.map(Into::into),
                Err(e) => http::Response::builder()
                    .header(CONTENT_TYPE, "text/plain")
                    .status(500)
                    .body(e.to_string().as_bytes().to_vec())
                    .unwrap()
                    .map(Into::into),
            }
        })
        .build()
        .expect("failed to create webview");

    let source = world.get_resource::<WebUISource>();
    if let Some(source) = source {
        webview.load_url(&source.url);
    }
    world.insert_non_send_resource(webview);
}

pub fn handle_source_change(webview: NonSendMut<WebView>, source: Res<WebUISource>) {
    if (source.is_changed()) {
        webview.load_url(&source.url);
    }
}

pub fn initial_keep_webview_fullscreen(
    mut resize_reader: EventReader<WindowResized>,
    webview: NonSendMut<WebView>,
    primary_window_entity: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_window = primary_window_entity.single();
    for resize_event in resize_reader.read() {
        let WindowResized {
            window,
            width,
            height,
        } = resize_event;
        if *window != primary_window {
            continue;
        }

        webview.set_bounds(wry::Rect {
            x: 0,
            y: 0,
            width: *width as u32,
            height: *height as u32,
        });
    }
}

pub fn keep_webview_fullscreen(
    webview: NonSendMut<WebView>,
    primary_window_entity: Query<(&Window), With<PrimaryWindow>>,
) {
    let (prim_window) = primary_window_entity.single();
    let (width, height) = (prim_window.width(), prim_window.height());
    webview.set_bounds(wry::Rect {
        x: 0,
        y: 0,
        width: width as u32,
        height: height as u32,
    });
}

fn get_core_ui_response(
    request: Request<Vec<u8>>,
) -> Result<http::Response<Vec<u8>>, Box<dyn std::error::Error>> {
    let path = request.uri().path();
    println!("ui path: {}", path);

    let blank = include_str!("../../ui/index.html");
    let blank = blank.as_bytes().to_vec();
    let response = http::Response::builder()
        .header(CONTENT_TYPE, "text/html")
        .status(200)
        .body(blank)
        .unwrap();
    Ok(response)
}
