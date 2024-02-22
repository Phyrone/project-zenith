use bevy::utils::default;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;

pub mod player_agent;

/*
async fn rtc() {
    let rtc_api = APIBuilder::new()
        .build();

    let connection = rtc_api.new_peer_connection(RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..default()
    }).await.unwrap();

    let channel = connection.create_data_channel("move", Some(RTCDataChannelInit {
        ordered: Some(true),
        max_retransmits: Some(30),
        ..default()
    })).await.unwrap();


    connection.on_data_channel(Box::new(|channel| {
        Box::pin(async move {

        })
    }));

    channel.on_message(Box::new(|msg| {
        Box::pin(async move {
            let data = msg.data;

        })
    }));
}

 */