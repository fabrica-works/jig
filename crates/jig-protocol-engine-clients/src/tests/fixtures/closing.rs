use super::*;
pub fn linked_settings() -> Settings {
    let identity = people::Identity {
        key: people::IdentityKey { provider: 0, subject: Box::from(b"owner".as_slice()) },
        login: Box::from(b"owner".as_slice()),
        name: Box::from(b"Owner".as_slice()),
    };
    Settings { link: Some(identity), ..settings() }
}
