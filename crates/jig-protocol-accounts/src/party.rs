//! A party's public LLM account sign-in registration (llm.md 5.2, 5.4).
use crate::installation;
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Writer};
use skein_oauth as oauth;
use skein_oauth_accounts as driver;

/// Public LLM account sign-in terms, translated by the host from its configuration.
/// The host resolves the endpoint transport and origins before building this input.
#[derive(Debug)]
pub struct Registration {
    pub issuer: Box<[u8]>,
    pub client: Box<[u8]>,
    pub scopes: Box<[Box<[u8]>]>,
    pub name_hint: Option<Box<[u8]>>,
    pub authorization: Box<[u8]>,
    pub token: Box<[u8]>,
    pub revocation: Option<Box<[u8]>>,
    pub keys: Box<[u8]>,
    pub endpoint_transport: driver::Transport,
    pub must_grant: Box<[Box<[u8]>]>,
    pub resource: Option<Box<[u8]>>,
    pub redirect_path: Box<[u8]>,
    pub redirect_port: Option<u16>,
}

/// Joins the provider sign-in scopes in OAuth's space-separated form.
#[must_use]
pub fn scopes(values: &[Box<[u8]>]) -> Box<[u8]> {
    let mut length = values.len().saturating_sub(1);
    for value in values {
        length = length.checked_add(value.len()).expect("configured scope bytes");
    }
    let mut out = Writer::new(length);
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.put(b" ").expect("measured scopes");
        }
        out.put(value).expect("measured scopes");
    }
    out.finish()
}

/// Builds the account sign-in from public configuration and host-resolved origins.
#[must_use]
pub fn party_account(
    configured: &Registration,
    endpoint: driver::Endpoint,
    origins: List<driver::Origin>,
    installation_id: &[u8; 16],
) -> driver::Account {
    let mut terms = installation::terms(installation_id);
    if let Some(hint) = &configured.name_hint {
        terms.parameters = Box::new([
            terms.parameters.first().expect("installation parameter").clone(),
            oauth::Parameter {
                name: b"agent_name_hint".as_slice().into(),
                value: hint.clone(),
                sent: oauth::Sent::NewRegistration,
            },
        ]);
    }
    terms.openid = Some(oauth::OpenId { issuer: configured.issuer.clone(), key_set: Some(configured.keys.clone()) });
    terms.issued_client_id = true;
    terms.varying_port = configured.redirect_port.is_none();
    terms.resource.clone_from(&configured.resource);
    terms.required_scopes = Some(scopes(&configured.must_grant));
    terms.revocation_endpoint.clone_from(&configured.revocation);
    let port = Decimal::of(u64::from(configured.redirect_port.unwrap_or(0)));
    let mut redirect = Writer::new(
        18_usize
            .checked_add(port.as_bytes().len())
            .expect("port bytes")
            .checked_add(configured.redirect_path.len())
            .expect("redirect bytes"),
    );
    redirect.put(b"http://127.0.0.1:").expect("measured redirect");
    redirect.put(port.as_bytes()).expect("measured redirect");
    redirect.put(b"/").expect("measured redirect");
    redirect.put(&configured.redirect_path).expect("measured redirect");
    driver::Account::SignIn {
        registration: oauth::Registration {
            authorization_url: configured.authorization.clone(),
            token_endpoint: configured.token.clone(),
            client_id: configured.client.clone(),
            redirect_uri: redirect.finish(),
            scope: scopes(&configured.scopes),
            wire: oauth::WireFormat::Form,
            client_secret: None,
            pkce_for_confidential: false,
            metadata_claim: None,
            terms: Some(terms),
        },
        endpoint: driver::Endpoint {
            address: endpoint.address,
            transport: match &configured.endpoint_transport {
                driver::Transport::Tls { server_name, trust } => {
                    driver::Transport::Tls { server_name: server_name.clone(), trust: trust.clone() }
                }
                driver::Transport::Plaintext => driver::Transport::Plaintext,
            },
        },
        keeper: driver::Keeper::Owner { kept: None },
        shared: false,
        origins,
    }
}

/// Reattaches the stored record to a party source before its first Want.
/// A missing or unreadable record remains an unloaded source.
pub fn restore(
    mut account: driver::Account,
    source: u32,
    value: Option<skein_lib::Secret>,
    records: &mut crate::records::Records,
    limits: &oauth::Limits,
) -> driver::Account {
    let record = match value {
        Some(value) => records.load(source, value.expose(), limits),
        None => {
            records.missing(source);
            None
        }
    };
    match &mut account {
        driver::Account::SignIn { keeper, .. } => *keeper = driver::Keeper::Owner { kept: record },
        driver::Account::HandedIn => unreachable!("party signed-in source uses its provider registration"),
    }
    account
}
