use crate::{
    Bounds, BoundsParts, Credential, CredentialParts, Hello, HelloParts, Hosting, HostingParts, ModelBound,
    ModelBoundParts, Phase, v1,
};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Secret, Writer};

pub const WORKER_A: &[u8] = b"worker-a";
pub const WORKER_B: &[u8] = b"worker-b";
pub const MODEL_SMALL: &[u8] = b"small";

pub const LIMITS: v1::Limits = v1::Limits {
    credential_name: 16,
    credential_secret: 32,
    model_bound_name: 16,
    bounds_models: 2,
    hello_workstreams: 2,
    hello_hosting: 2,
};

pub fn encode(hello: &Hello) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(hello.measure()).expect("small hello"));
    hello.encode(&mut writer).expect("measured room");
    writer.finish()
}

pub fn credential(name: &[u8], secret: &[u8]) -> Credential {
    Credential::new(&LIMITS, CredentialParts { name: name.into(), secret: Secret::new(secret.into()), payload: 1 })
        .expect("small credential")
}

pub fn model() -> ModelBound {
    ModelBound::new(&LIMITS, ModelBoundParts { name: MODEL_SMALL.into(), window: 8192, output: 1024 })
        .expect("small model")
}

pub fn hello_fullest() -> Hello {
    let mut models = List::with_capacity(2);
    models.push(model()).expect("first model slot");
    models.push(model()).expect("second model slot");
    let bounds = Bounds::new(
        &LIMITS,
        BoundsParts {
            models,
            turns: 40,
            spend: 100,
            time: Duration::from_secs(300),
            waiting: Duration::from_secs(60),
            checks: Duration::from_secs(20),
            message_bytes: 32,
            inbox: 2,
            unacknowledged: 2,
        },
    )
    .expect("small agent bounds");
    let mut workstreams = List::with_capacity(2);
    workstreams.push(7).expect("first workstream slot");
    workstreams.push(8).expect("second workstream slot");
    let mut hosting = List::with_capacity(2);
    for phase in [Phase::Active, Phase::Answered] {
        hosting
            .push(Hosting::new(&LIMITS, HostingParts { task: 3, attempt: 4, phase }).expect("hosting record"))
            .expect("two hosting slots");
    }
    Hello::new(&LIMITS, HelloParts { slots: 2, stop_bound: Duration::from_secs(95), workstreams, hosting, bounds })
        .expect("full hello within small list limits")
}

pub fn hello_smallest() -> Hello {
    // Three empty counts, plus the hello's and agent bounds' fixed fields.
    Hello::decode(&LIMITS, &mut skein_lib::Reader::new(&[0; 72])).expect("smallest hello")
}
