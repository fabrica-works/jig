//! Total client request translations (protocol/client.md, sections 3 and 5).
pub(crate) mod asks;
use jig_protocol_client_wire::v5 as w;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Untranslatable {
    pub path: w::Path,
}
