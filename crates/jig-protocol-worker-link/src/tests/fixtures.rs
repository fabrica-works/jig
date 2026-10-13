use crate::{Credential, Limits};
use skein_lib::{Duration, Secret};
pub const WORKER_A: &[u8] = b"worker-a";
pub const SECRET_A: &[u8] = b"secret-a";
pub const PAYLOAD: u32 = 3;
pub const SEED_ONE: u64 = 1;
pub const SIZES: jig_protocol_worker_wire::sizes::Sizes = jig_protocol_worker_wire::sizes::Sizes {
    charter: 16,
    transcript: 16,
    turn: 16,
    message: 16,
    sender: 16,
    call_name: 16,
    tool_name: 16,
    call_input: 16,
    answer: 16,
    result: 16,
    detail: 16,
    grant: 16,
    account_id: 16,
    name: 16,
    secret: 16,
    model_name: 16,
    items: 16,
    left: 16,
    saved: 16,
    evidence: 16,
    resource: 16,
    slots: 1,
    calls: 1,
    turns: 1,
    inbox: 1,
    answered: 1,
    models: 1,
    workstreams: 1,
    accounts: 1,
    credentials: 1,
    facts: 16,
};

pub const LIMITS: Limits = Limits {
    sizes: SIZES,
    chunk: 64,
    handshake: Duration::from_secs(1),
    ping: Duration::from_secs(5),
    silence: Duration::from_secs(15),
    stall: Duration::from_secs(30),
    backoff: Duration::from_millis(100),
    ceiling: Duration::from_millis(1600),
    proved: Duration::from_secs(2),
    grace: Duration::from_secs(10),
    busy: Duration::from_millis(200),
};
pub fn credential() -> Credential {
    Credential { name: WORKER_A.into(), secret: Secret::new(SECRET_A.into()), payload: PAYLOAD }
}
