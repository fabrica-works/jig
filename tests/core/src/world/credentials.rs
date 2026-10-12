//! Shared credential-bearing configuration for the core world.
use super::root;
use crate::fixtures::WORKSPACE_CREDENTIAL;

#[must_use]
pub fn with_workspace_credential(seed: u64) -> (root::Config, root::Limits) {
    let (mut config, limits) = crate::charters::two_charters(seed);
    config.first.resources[0].credential = Some(WORKSPACE_CREDENTIAL);
    (config, limits)
}
