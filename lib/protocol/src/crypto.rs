use std::sync::Arc;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::client::ResolvesClientCert;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use rustls::{DigitallySignedStruct, DistinguishedName, Error, SignatureScheme};

#[derive(Debug)]
pub struct UnsafeServerCertVerifier;

impl ServerCertVerifier for UnsafeServerCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![SignatureScheme::ED25519]
    }
}

#[derive(Debug)]
pub struct AClientCertVerifier;

impl ClientCertVerifier for AClientCertVerifier {
    fn offer_client_auth(&self) -> bool {
        true
    }

    fn client_auth_mandatory(&self) -> bool {
        false
    }

    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        return &[];
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        now: UnixTime,
    ) -> Result<ClientCertVerified, Error> {
        Ok(ClientCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![SignatureScheme::ED25519]
    }
}

#[derive(Debug)]
pub struct AServerCertResolver;

impl ResolvesServerCert for AServerCertResolver {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        client_hello.server_name();
        todo!()
    }
}

#[derive(Debug)]
pub struct AClientCertResolver;

impl ResolvesClientCert for AClientCertResolver {
    fn resolve(
        &self,
        root_hint_subjects: &[&[u8]],
        sigschemes: &[SignatureScheme],
    ) -> Option<Arc<CertifiedKey>> {
        root_hint_subjects;
        None
    }

    fn has_certs(&self) -> bool {
        false
    }
}

pub mod test {
    use rcgen::{
        CertificateParams, CertificateSigningRequest, CertifiedKey, DnType, KeyPair, PKCS_ED25519,
    };
    use time::OffsetDateTime;

    #[tokio::test]
    async fn test_server() {
        let CertifiedKey {
            cert: ca_cert,
            key_pair: ca_key_pair,
        } = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();

        let keypair = KeyPair::generate_for(&PKCS_ED25519).expect("Failed to generate keypair");
        println!("{}", keypair.serialize_pem());
        let mut cert = CertificateParams::default();
        let mut distinguished_name = rcgen::DistinguishedName::new();
        distinguished_name.push(DnType::CommonName, "client:1iotlogci4ah0");
        distinguished_name.push(DnType::OrganizationName, "private");
        cert.distinguished_name = distinguished_name;
        cert.not_after = OffsetDateTime::now_utc() + time::Duration::hours(24);
        cert.not_before = OffsetDateTime::now_utc();

        //cert.subject_alt_names = vec![SanType::URI(Ia5String::from_str("user://1A7C5836-303C-446F-9395-90A0E5C60D2D").expect("Failed to parse URI"))];
        let cert = cert
            .signed_by(&keypair, &ca_cert, &ca_key_pair)
            .expect("Failed to sign certificate");
        println!("{}", cert.pem());
        println!("{}", ca_cert.pem());

        //let server = quinn::Endpoint::server()
    }
}
