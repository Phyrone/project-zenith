use std::fmt::Debug;

use bevy::prelude::Component;
use bytes::Bytes;
use rustls::client::danger::ServerCertVerifier;
use tonic::codegen::Body;

mod client;
mod crypto;
mod encoding;
mod local;
pub mod proto;
mod server;

const DEFAULT_PORT: u16 = 26381;
const MAX_MESSAGE_SIZE: usize = 32 * 1024 * 1024;
const COMPRESSION_THRESHOLD: usize = 1024;

pub enum UniChannelOut {
    Sync(UniChannelOutSync),
    //TODO RPC
}

pub struct UniChannelOutSync {
    pub channel: u8,
    pub data: Bytes,
}
