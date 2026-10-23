//! Startup construction at the provider's configuration paths (client.md, 18).
use super::Provider;
use crate::{Path, Problem, Segment, What};
use alloc::boxed::Box;
use skein_lib::{List, Secret, bytes};
use skein_oauth_issuer::{Client, Error, Issuer, Settings};

/// The application's own addresses; never part of jig's configuration.
#[derive(Debug)]
pub struct ProviderUrls {
    pub issuer: Box<[u8]>,
    pub public_origin: Box<[u8]>,
    pub authorization_endpoint: Box<[u8]>,
    pub token_endpoint: Box<[u8]>,
    pub userinfo_endpoint: Box<[u8]>,
    pub jwks_uri: Box<[u8]>,
}
/// Files have already been read privately by the application's startup.
pub fn provider_issuer(
    provider: &Provider,
    files: &[(Box<[u8]>, Secret)],
    urls: ProviderUrls,
    limits: &jig_protocol_engine_clients::Limits,
) -> Result<Issuer, Problem> {
    let path = Path::root().member(b"clients").member(b"provider");
    let bounds = jig_protocol_engine_clients::provider_limits(limits);
    let key = super::provider_key(contents(files, &provider.key, &path.member(b"key"))?.expose())?;
    let mut probe = List::with_capacity(1);
    probe
        .push(Client {
            id: Box::from(b"probe".as_slice()),
            secret: Secret::new(Box::from(b"probe".as_slice())),
            redirect_uri: urls.public_origin.clone(),
        })
        .expect("one probe");
    if let Err(error) = Issuer::new(settings(&urls, probe, key.clone()), bounds) {
        return Err(Problem { path: path.clone(), what: reason(error) });
    }
    let mut clients = List::with_capacity(bounds.clients);
    if provider.clients.is_empty() || provider.clients.len() > usize::try_from(bounds.clients).expect("u32 fits") {
        return Err(Problem { path: path.member(b"clients"), what: What::Bound });
    }
    for (index, configured) in provider.clients.iter().enumerate() {
        let at = path.member(b"clients").child(Segment::Index(u32::try_from(index).expect("bounded clients")));
        let secret = contents(files, &configured.secret, &at.member(b"secret"))?;
        let secret = crate::secret(bytes::copy_of(secret.expose()));
        if configured.id.is_empty() || configured.id.len() > usize::try_from(bounds.client_bytes).expect("u32 fits") {
            return Err(Problem { path: at.member(b"id"), what: What::Bound });
        }
        for byte in &configured.id {
            if !(0x21..=0x7e).contains(byte) {
                return Err(Problem { path: at.member(b"id"), what: What::Form });
            }
        }
        for previous in &clients {
            let previous: &Client = previous;
            if previous.id == configured.id {
                return Err(Problem { path: at.member(b"id"), what: What::Unique });
            }
        }
        if secret.is_empty() || secret.len() > usize::try_from(bounds.client_bytes).expect("u32 fits") {
            return Err(Problem { path: at.member(b"secret"), what: What::Bound });
        }
        // Use the issuer's URL rules too; no second reading of its address grammar.
        let mut one = List::with_capacity(1);
        one.push(Client {
            id: configured.id.clone(),
            secret: secret.clone(),
            redirect_uri: configured.redirect_uri.clone(),
        })
        .expect("one client");
        if let Err(error) = Issuer::new(settings(&urls, one, key.clone()), bounds) {
            let member = match error {
                Error::Address | Error::Limits => at.member(b"redirect_uri"),
                Error::Client => at,
                Error::Key => path.member(b"key"),
                Error::Claims | Error::Sign => unreachable!("issuer construction checks no claims and signs nothing"),
            };
            return Err(Problem { path: member, what: reason(error) });
        }
        clients
            .push(Client { id: configured.id.clone(), secret, redirect_uri: configured.redirect_uri.clone() })
            .expect("client count checked");
    }
    match Issuer::new(settings(&urls, clients, key), bounds) {
        Ok(issuer) => Ok(issuer),
        Err(error) => Err(Problem { path, what: reason(error) }),
    }
}
fn settings(urls: &ProviderUrls, clients: List<Client>, key: Secret) -> Settings {
    Settings {
        issuer: urls.issuer.clone(),
        authorization_endpoint: urls.authorization_endpoint.clone(),
        token_endpoint: urls.token_endpoint.clone(),
        userinfo_endpoint: urls.userinfo_endpoint.clone(),
        jwks_uri: urls.jwks_uri.clone(),
        clients,
        key,
    }
}
fn reason(error: Error) -> What {
    match error {
        Error::Limits => What::Bound,
        Error::Address | Error::Client => What::Form,
        Error::Key => What::ProviderKey { generate: "openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:3072" },
        Error::Claims | Error::Sign => unreachable!("issuer construction checks no claims and signs nothing"),
    }
}
fn contents(files: &[(Box<[u8]>, Secret)], name: &[u8], at: &Path) -> Result<Secret, Problem> {
    for (file, value) in files {
        if file.as_ref() == name {
            return Ok(value.clone());
        }
    }
    Err(Problem { path: at.clone(), what: What::Unreadable })
}
