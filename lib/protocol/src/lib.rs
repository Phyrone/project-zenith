use std::fmt::Debug;

use rustls::client::danger::ServerCertVerifier;
use tonic::codegen::Body;

mod crypto;
pub mod proto;
mod stream_encoding;

const DEFAULT_PORT: u16 = 26381;
//Messages after packetid are limited to 16MB
const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024;
