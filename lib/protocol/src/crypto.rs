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

mod utils {
    use error_stack::ResultExt;
    use rcgen::{
        Certificate, CertificateParams, CertificateSigningRequest, DnType, ExtendedKeyUsagePurpose,
        IsCa, KeyPair, OtherNameValue, SanType, SerialNumber, PKCS_ED25519,
    };
    use thiserror::Error;

    const SESSION_CERTIFICATE_NAME: &str = "p-zenith-account-ownership-attestation";
    const ALT_NAME_TYPE: [u64; 2] = [0x000815, 0x42];

    #[derive(Debug, Error)]
    pub enum CreateSessionError {
        #[error("could not generate keypair")]
        GenerateKeyPair,

        #[error("could not self sign certificate")]
        SelfSignCertificate,

        #[error("could not sign certificate")]
        SignCertificate,

        #[error("could not create sign request")]
        CreateRequestError,
    }

    pub struct SessionCreation {
        keypair: KeyPair,
        certificate: CertificateParams,
    }

    impl SessionCreation {
        pub fn new(account_id: u64) -> error_stack::Result<Self, CreateSessionError> {
            let keypair = KeyPair::generate_for(&PKCS_ED25519)
                .change_context(CreateSessionError::GenerateKeyPair)?;
            let mut params = CertificateParams::new(vec![])
                .expect("cannot fail since no alt params are provided");
            params
                .distinguished_name
                .push(DnType::CommonName, SESSION_CERTIFICATE_NAME);
            params.is_ca = IsCa::NoCa;
            params.not_after = time::OffsetDateTime::now_utc() + time::Duration::minutes(15);
            params.not_before = time::OffsetDateTime::now_utc();
            params.use_authority_key_identifier_extension = true;
            //TODO to something with the serial number
            params.serial_number = Some(SerialNumber::from_slice(&[0xFE, 0xED, 0xBE, 0xEF]));
            params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
            params.subject_alt_names = vec![SanType::OtherName((
                ALT_NAME_TYPE.to_vec(),
                OtherNameValue::Utf8String("".to_string()),
            ))];

            Ok(Self {
                keypair,
                certificate: params,
            })
        }

        pub fn to_offline_session(
            mut self,
        ) -> error_stack::Result<(Certificate, KeyPair), CreateSessionError> {
            self.certificate.not_after =
                time::OffsetDateTime::now_utc() + time::Duration::minutes(15);
            self.certificate.not_before = time::OffsetDateTime::now_utc();

            let certificate = self
                .certificate
                .self_signed(&self.keypair)
                .change_context(CreateSessionError::SelfSignCertificate)?;
            Ok((certificate, self.keypair))
        }
        pub fn create_sign_request(
            &self,
        ) -> error_stack::Result<CertificateSigningRequest, CreateSessionError> {
            self.certificate
                .serialize_request(&self.keypair)
                .change_context(CreateSessionError::SelfSignCertificate)
        }
    }
}

pub mod test {
    use crate::crypto::utils::SessionCreation;

    #[tokio::test]
    async fn test_server() {
        let session_creation = SessionCreation::new(13).expect("could not create session");
        let (certificate, key) = session_creation
            .to_offline_session()
            .expect("could not create offline session");
        println!("{:?}", certificate.pem());
        println!("{:?}", key.serialize_pem());

        //let server = quinn::Endpoint::server()
    }
}
