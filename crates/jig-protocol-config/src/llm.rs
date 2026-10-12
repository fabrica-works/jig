//! Engine endpoint, model and account values (llm.md, 4 and 5).
use crate::{Address, Limits, Path, Problems, Segment, What, check, read};
use alloc::boxed::Box;
use jig_domain_core::Model;
use skein_json::Document;
use skein_lib::{Duration, List};
#[derive(Debug)]
pub struct Llm {
    pub endpoints: Box<[Endpoint]>,
    pub models: Box<[Model]>,
    pub accounts: Box<[Account]>,
    pub records: Box<[u8]>,
}
#[derive(Debug)]
pub struct Endpoint {
    pub name: Box<[u8]>,
    pub number: u32,
    pub dialect: u32,
    pub account: u32,
    pub provider: Box<[u8]>,
    pub address: Address,
    pub transport: Transport,
    pub headers: Box<[Header]>,
    pub connect: Duration,
    pub handshake: Duration,
}
#[derive(Debug)]
pub struct Header {
    pub name: Box<[u8]>,
    pub value: Box<[u8]>,
}
#[derive(Debug)]
pub enum Transport {
    Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> },
    Plaintext,
}
#[derive(Debug)]
pub struct Account {
    pub number: u32,
    pub source: Source,
}
#[derive(Debug)]
pub enum Source {
    SignIn(SignIn),
    HandedIn { file: Box<[u8]> },
}
#[derive(Debug)]
pub struct SignIn {
    pub client: Box<[u8]>,
    pub secret: Option<Box<[u8]>>,
    pub authorization: Box<[u8]>,
    pub token: Box<[u8]>,
    pub endpoint: Address,
    pub transport: Transport,
    pub redirect: Box<[u8]>,
    pub scope: Box<[u8]>,
}
pub(crate) fn parse(
    document: &Document,
    at: u32,
    path: &Path,
    limits: &Limits,
    problems: &mut Problems,
) -> Option<Llm> {
    let mut object = check::object(document, at, path, problems)?;
    let max = limits.core.accounts.accounts;
    let mut endpoints = List::<Endpoint>::with_capacity(document.len());
    if let Some(items) = check::list(&mut object, document, b"endpoints", document.len(), path, problems) {
        for (index, at) in items.iter().enumerate() {
            let path =
                path.member(b"endpoints").child(Segment::Index(u32::try_from(index).expect("list bounded by u32")));
            if let Some(value) = endpoint(document, *at, &path, problems) {
                for previous in &endpoints {
                    if previous.name == value.name {
                        problems.push(path.member(b"name"), What::Unique);
                    }
                    if previous.number == value.number {
                        problems.push(path.member(b"number"), What::Unique);
                    }
                }
                endpoints.push(value).expect("input count bounded");
            }
        }
    }
    let mut accounts = List::<Account>::with_capacity(max);
    if let Some(items) = check::list(&mut object, document, b"accounts", max, path, problems) {
        for (index, at) in items.iter().enumerate() {
            let path =
                path.member(b"accounts").child(Segment::Index(u32::try_from(index).expect("list bounded by u32")));
            if let Some(value) = account(document, *at, &path, problems) {
                for previous in &accounts {
                    if previous.number == value.number {
                        problems.push(path.member(b"number"), What::Unique);
                        break;
                    }
                }
                accounts.push(value).expect("input count bounded");
            }
        }
    }
    for (index, endpoint) in endpoints.iter().enumerate() {
        let mut found = false;
        for account in &accounts {
            if account.number == endpoint.account {
                found = true;
                break;
            }
        }
        if !found {
            problems.push(
                path.member(b"endpoints")
                    .child(Segment::Index(u32::try_from(index).expect("bounded")))
                    .member(b"account"),
                What::Refers,
            );
        }
    }
    let mut models = List::<Model>::with_capacity(document.len());
    if let Some(items) = check::list(&mut object, document, b"models", document.len(), path, problems) {
        for (index, at) in items.iter().enumerate() {
            let path = path.member(b"models").child(Segment::Index(u32::try_from(index).expect("list bounded by u32")));
            if let Some(value) = model(document, *at, &path, endpoints.as_slice(), problems) {
                for previous in &models {
                    if previous.name == value.name {
                        problems.push(path.member(b"name"), What::Unique);
                        break;
                    }
                }
                models.push(value).expect("input count bounded");
            }
        }
    }
    let records = check::text(&mut object, document, b"records", path, problems);
    object.finish(document, path, problems);
    Some(Llm {
        endpoints: endpoints.into_boxed(),
        models: models.into_boxed(),
        accounts: accounts.into_boxed(),
        records: records?,
    })
}
fn endpoint(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Endpoint> {
    let mut object = check::object(document, at, path, problems)?;
    let name = check::text(&mut object, document, b"name", path, problems);
    let number = check::count(&mut object, document, b"number", path, problems);
    let dialect = check::count(&mut object, document, b"dialect", path, problems);
    let account = check::count(&mut object, document, b"account", path, problems);
    let provider = check::text(&mut object, document, b"provider", path, problems);
    let address = check::address(&mut object, document, b"address", path, problems);
    if let Some(address) = &address {
        network_address(address, &path.member(b"address"), problems);
    }
    let transport = transport(&mut object, document, path, b"address", address.as_ref(), problems);
    let headers = headers(&mut object, document, path, problems);
    let connect = check::duration(&mut object, document, b"connect_ms", path, problems);
    let handshake = check::duration(&mut object, document, b"handshake_ms", path, problems);
    object.finish(document, path, problems);
    Some(Endpoint {
        name: name?,
        number: number?,
        dialect: dialect?,
        account: account?,
        provider: provider?,
        address: address?,
        transport: transport?,
        headers: headers.into_boxed(),
        connect: connect?,
        handshake: handshake?,
    })
}
fn transport(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    address_name: &[u8],
    address: Option<&Address>,
    problems: &mut Problems,
) -> Option<Transport> {
    let kind = check::text(object, document, b"transport", path, problems)?;
    match kind.as_ref() {
        b"plaintext" => {
            if let Some(address) = address {
                check::local(address, &path.member(address_name), problems);
            }
            Some(Transport::Plaintext)
        }
        b"tls" => {
            let server_name = check::text(object, document, b"server_name", path, problems);
            let trust = optional_text(object, document, b"trust", path, problems);
            Some(Transport::Tls { server_name: server_name?, trust })
        }
        _ => {
            problems.push(path.member(b"transport"), What::Form);
            None
        }
    }
}
fn optional_text(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[u8]>> {
    let at = object.member(document, name)?;
    match read::text(document, at, 4_096) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.member(name), what);
            None
        }
    }
}
fn headers(object: &mut read::Object, document: &Document, path: &Path, problems: &mut Problems) -> List<Header> {
    let mut result = List::with_capacity(document.len());
    if let Some(items) = check::list(object, document, b"headers", document.len(), path, problems) {
        for (index, at) in items.iter().enumerate() {
            let path = path.member(b"headers").child(Segment::Index(u32::try_from(index).expect("bounded list")));
            if let Some(mut object) = check::object(document, *at, &path, problems) {
                let name = check::text(&mut object, document, b"name", &path, problems);
                let value = check::text(&mut object, document, b"value", &path, problems);
                object.finish(document, &path, problems);
                if let Some(name) = name
                    && let Some(value) = value
                {
                    result.push(Header { name, value }).expect("input count bounded");
                }
            }
        }
    }
    result
}
fn account(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Account> {
    let mut object = check::object(document, at, path, problems)?;
    let number = check::count(&mut object, document, b"number", path, problems);
    let mut parsed_source = None;
    if let Some(at) = object.required(document, b"source", path, problems) {
        parsed_source = source(document, at, &path.member(b"source"), problems);
    }
    object.finish(document, path, problems);
    Some(Account { number: number?, source: parsed_source? })
}
fn source(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Source> {
    let mut object = check::object(document, at, path, problems)?;
    let handed = object.member(document, b"handed_in");
    let signed = object.member(document, b"sign_in");
    let mut result = None;
    if handed.is_some() == signed.is_some() {
        problems.push(path.clone(), What::Form);
    } else if let Some(at) = handed {
        let path = path.member(b"handed_in");
        if let Some(mut object) = check::object(document, at, &path, problems) {
            let file = check::text(&mut object, document, b"file", &path, problems);
            object.finish(document, &path, problems);
            if let Some(file) = file {
                result = Some(Source::HandedIn { file });
            }
        }
    } else if let Some(at) = signed
        && let Some(value) = sign_in(document, at, &path.member(b"sign_in"), problems)
    {
        result = Some(Source::SignIn(value));
    }
    object.finish(document, path, problems);
    result
}
fn sign_in(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<SignIn> {
    let mut object = check::object(document, at, path, problems)?;
    let client = check::text(&mut object, document, b"client", path, problems);
    let secret = optional_text(&mut object, document, b"secret", path, problems);
    let authorization = check::text(&mut object, document, b"authorization", path, problems);
    let token = check::text(&mut object, document, b"token", path, problems);
    let endpoint = check::address(&mut object, document, b"endpoint", path, problems);
    if let Some(address) = &endpoint {
        network_address(address, &path.member(b"endpoint"), problems);
    }
    let transport = transport(&mut object, document, path, b"endpoint", endpoint.as_ref(), problems);
    let redirect = check::text(&mut object, document, b"redirect", path, problems);
    let scope = check::text(&mut object, document, b"scope", path, problems);
    object.finish(document, path, problems);
    Some(SignIn {
        client: client?,
        secret,
        authorization: authorization?,
        token: token?,
        endpoint: endpoint?,
        transport: transport?,
        redirect: redirect?,
        scope: scope?,
    })
}
fn model(document: &Document, at: u32, path: &Path, endpoints: &[Endpoint], problems: &mut Problems) -> Option<Model> {
    let mut object = check::object(document, at, path, problems)?;
    let name = check::text(&mut object, document, b"name", path, problems);
    let endpoint = check::text(&mut object, document, b"endpoint", path, problems);
    let window = check::count(&mut object, document, b"window", path, problems);
    let output = check::count(&mut object, document, b"output", path, problems);
    let mut prices = None;
    if let Some(at) = object.required(document, b"prices", path, problems) {
        prices = price(document, at, &path.member(b"prices"), problems);
    }
    object.finish(document, path, problems);
    let mut found = None;
    if let Some(name) = endpoint {
        for endpoint in endpoints {
            if endpoint.name == name {
                found = Some(endpoint);
                break;
            }
        }
        if found.is_none() {
            problems.push(path.member(b"endpoint"), What::Refers);
        }
    }
    let endpoint = found?;
    let mut model = prices?;
    model.name = name?;
    model.dialect = endpoint.dialect;
    model.account = endpoint.account;
    model.endpoint = endpoint.number;
    model.window = window?;
    model.output = output?;
    Some(model)
}
fn price(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Model> {
    let mut object = check::object(document, at, path, problems)?;
    let input_price = check::integer(&mut object, document, b"input", path, problems);
    let cached_price = check::integer(&mut object, document, b"cached", path, problems);
    let output_price = check::integer(&mut object, document, b"output", path, problems);
    let price_unit = check::count(&mut object, document, b"unit", path, problems);
    let mut cache_write_price = None;
    if let Some(at) = object.member(document, b"cache_write") {
        match read::integer(document, at, u64::MAX) {
            Ok(value) => cache_write_price = Some(value),
            Err(what) => problems.push(path.member(b"cache_write"), what),
        }
    }
    object.finish(document, path, problems);
    Some(Model {
        name: Box::default(),
        dialect: 0,
        account: 0,
        endpoint: 0,
        window: 0,
        output: 0,
        input_price: input_price?,
        cached_price: cached_price?,
        cache_write_price,
        output_price: output_price?,
        price_unit: price_unit?,
    })
}

// Decision 4: LLM and OAuth dial socket addresses, with host resolution at startup.
fn network_address(address: &Address, path: &Path, problems: &mut Problems) {
    match address {
        Address::Addr(skein_io::Addr::InProcess(_)) => problems.push(path.clone(), What::Form),
        Address::Addr(skein_io::Addr::Ip(_)) | Address::Host { .. } => {}
    }
}
