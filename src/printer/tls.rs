/// Some TLS tools, only "AcceptAnyCert" verifier for mqtt
/// since Bambu printer TLS certs are self-signed. Ideally,
/// I'd verify only the CA to match bambu printers but that's
/// too much work I'm willing to put in for now.

#[derive(Debug)]
pub struct AcceptAnyCert;

impl rustls::client::danger::ServerCertVerifier for AcceptAnyCert {
  fn verify_server_cert(
    &self,
    _: &rustls::pki_types::CertificateDer,
    _: &[rustls::pki_types::CertificateDer],
    _: &rustls::pki_types::ServerName,
    _: &[u8],
    _: rustls::pki_types::UnixTime,
  ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
    Ok(rustls::client::danger::ServerCertVerified::assertion())
  }
  fn verify_tls12_signature(
    &self,
    _: &[u8],
    _: &rustls::pki_types::CertificateDer,
    _: &rustls::DigitallySignedStruct,
  ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
    Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
  }
  fn verify_tls13_signature(
    &self,
    _: &[u8],
    _: &rustls::pki_types::CertificateDer,
    _: &rustls::DigitallySignedStruct,
  ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
    Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
  }
  fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
    rustls::crypto::ring::default_provider()
      .signature_verification_algorithms
      .supported_schemes()
  }
}
