use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use crossbeam::sync::Parker;

use quinn::{ClientConfig, Endpoint, ServerConfig, VarInt};
use quinn::crypto::rustls::QuicClientConfig;
use rcgen::{CertificateParams, KeyPair, PKCS_ED25519};
use rustls::{DigitallySignedStruct, Error, SignatureScheme};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task::JoinSet;
use tokio::time::sleep;

fn generate_server_config() -> ServerConfig {
    let key_pair = KeyPair::generate_for(&PKCS_ED25519)
        .expect("Failed to generate key pair");
    let params = CertificateParams::new(vec!["localhost".into()])
        .expect("Failed to create certificate params");

    let cert = params.self_signed(&key_pair)
                     .expect("Failed to create self-signed certificate");

    let pk = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_pair.serialize_der()));

    let mut server_config = ServerConfig::with_single_cert(vec![cert.der().clone()], pk)
        .expect("Failed to create server config");
    server_config.migration(true);
    server_config
}

#[derive(Clone, Debug)]
struct NoCertificateVerifier;

impl ServerCertVerifier for NoCertificateVerifier {
    fn verify_server_cert(&self, end_entity: &CertificateDer<'_>, intermediates: &[CertificateDer<'_>], server_name: &ServerName<'_>, ocsp_response: &[u8], now: UnixTime) -> Result<ServerCertVerified, Error> {
        return Ok(ServerCertVerified::assertion());
    }

    fn verify_tls12_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error>
    {
        return Ok(HandshakeSignatureValid::assertion());
    }

    fn verify_tls13_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> {
        return Ok(HandshakeSignatureValid::assertion());
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        return vec![SignatureScheme::ED25519];
    }
}

fn create_client_config() -> ClientConfig {
    let verifier = Arc::new(NoCertificateVerifier);
    let tls_client_config = rustls::ClientConfig::builder().dangerous()
                                                           .with_custom_certificate_verifier(verifier)
                                                           .with_no_client_auth();
    let tls_client_config = Arc::new(tls_client_config);
    let client_config = QuicClientConfig::try_from(tls_client_config)
        .expect("Failed to create client config");

    let client_config = ClientConfig::new(Arc::new(client_config));
    client_config
}

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider().install_default().expect("Failed to install ring crypto provider");

    let server_address: SocketAddr = "127.0.0.1:4242".parse().expect("cannot parse server address");

    let server = Endpoint::server(generate_server_config(), server_address.clone())
        .expect("Failed to create server endpoint");

    let mut client = Endpoint::client("0.0.0.0:0".parse().expect("cannot parse client address"))
        .expect("Failed to create client endpoint");
    client.set_default_client_config(create_client_config());
    let client_task = tokio::spawn(clientside(client, server_address));
    let server_task = tokio::spawn(server_side(server));

    tokio::select! {
        res = client_task => {
            res.expect("Client task failed");
            println!("Client task finished");
        }
        res = server_task => {
            res.expect("Server task failed");
            println!("Server task finished");
        }
    }
}

async fn server_side(server: Endpoint) {
    let serverside_connection = server
        .accept()
        .await
        .expect("failed to accept connection")
        .await
        .expect("failed to establish connection");
    serverside_connection.set_max_concurrent_uni_streams(VarInt::from_u32(16*1024));
    println!("[server] Connection established");
    let mut join_set = JoinSet::new();

    while let Ok(mut accpeted) = serverside_connection.accept_uni().await {
        join_set.spawn(async move {
            while let Ok(i) = accpeted.read_u8().await {
                // do nothing
            }
        });
    }
    
    drop(serverside_connection);
}

async fn clientside(client: Endpoint, server_address: SocketAddr) {
    let clientside_connection = client
        .connect(server_address, "localhost")
        .expect("failed to connect to server")
        .await
        .expect("failed to establish connection");
    let id  = clientside_connection.stable_id();
    let packet_push_qeue  = crossbeam::queue::ArrayQueue::new(16*1024);
    packet_push_qeue.push(0);
    let parker = Parker::new();
    
    
    

    
    
    println!("[client] Connection established {}", id);
    
    
    let mut tasks = JoinSet::new();
    for i in 0..1_000_000 {
        let mut stream = clientside_connection
            .open_uni()
            .await
            .expect("failed to open stream");
        tasks.spawn(async move {
            stream.write_u8(0).await.expect("failed to write to stream");
            stream.finish().expect("failed to finish stream");
        });
    }
}
