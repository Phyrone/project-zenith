use std::fmt::Debug;

use bevy::prelude::Component;
use bytes::Bytes;
use rustls::client::danger::ServerCertVerifier;
use tonic::codegen::Body;

mod client;
mod crypto;
mod encoding;
pub mod proto;
mod server;
mod transport;

const DEFAULT_PORT: u16 = 26381;
const MAX_MESSAGE_SIZE: usize = 32 * 1024 * 1024;
const COMPRESSION_THRESHOLD: usize = 1024;
