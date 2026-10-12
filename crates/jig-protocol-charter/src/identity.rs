//! Run identities derived at assignment (protocol/channel.md, section 7).
use sha2::{Digest, Sha256};

/// The first sixteen bytes of the deployment, task and conversation's SHA-256.
#[must_use]
pub fn run_identity(deployment: &[u8; 16], task: u64, conversation: u64) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(deployment);
    digest.update(task.to_be_bytes());
    digest.update(conversation.to_be_bytes());
    let bytes = digest.finalize();
    <[u8; 16]>::try_from(bytes.get(..16).expect("SHA-256 holds sixteen bytes")).expect("identity size")
}
