//! Sign in providers are configured by the application and secrets stay below.
//! Signing transitions land in jig 20.4; client/domain/client.md, 12.
use crate::contract::Text;
/// A configured provider number and display name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Provider {
    pub number: u16,
    pub name: Text,
}
