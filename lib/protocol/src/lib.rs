use std::fmt::Debug;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use enum_assoc::Assoc;
use quinn::ClientConfig;
use rustls::client::danger::ServerCertVerifier;
use tonic::body::BoxBody;
use tonic::codegen::{Body, StdError};
use tonic::codegen::http::{HeaderMap, Request};

use crate::proto::service::lobby::server::service_lobby_serverlist_client::ServiceLobbyServerlistClient;

mod crypto;
pub mod proto;
mod stream_encoding;

const DEFAULT_PORT: u16 = 26381;


pub trait PacketTransport{
    
}

pub trait EndpointTransport{
    
    
}
