//! Stream and remaining-document fixtures (protocol/client.md, 3 and 7).
use super::*;

pub fn stream_limits() -> Limits {
    let mut limits = CEILINGS;
    limits.owned_record = 32;
    limits.hello_served = 2;
    limits.run_text_text = 32;
    limits.call_identity_id = 32;
    limits.current_call_tool = 32;
    limits.current_call_input = 32;
    limits.call_tool = 32;
    limits.call_input = 32;
    limits.call_outcome_some = 32;
    limits.deployment_capacity = 2;
    check_limits(&limits).expect("stream fixture limits below ceilings");
    limits
}

pub fn wire_family(limits: &Limits) -> Owner {
    Owner::Family(Family::new(limits, FamilyParts { number: u16::MAX }).expect("whole family identity"))
}

pub fn served(limits: &Limits, owner: Owner) -> Served {
    Served::new(limits, ServedParts { owner, first: u32::MAX, last: u32::MAX }).expect("whole served range")
}

pub fn hello(limits: &Limits) -> Hello {
    let mut owners = List::with_capacity(2);
    owners.push(served(limits, Owner::Jig)).expect("first owner");
    owners.push(served(limits, wire_family(limits))).expect("second owner");
    Hello::new(
        limits,
        HelloParts { stream: [255; 16], served: owners, heartbeat: skein_lib::Duration::from_nanos(u64::MAX) },
    )
    .expect("two owners in hello")
}

pub fn stream_owned(limits: &Limits) -> Owned {
    Owned::new(limits, OwnedParts { owner: wire_family(limits), version: u32::MAX, record: bytes(limits.owned_record) })
        .expect("record at configured ceiling")
}

pub fn current_call(limits: &Limits) -> CurrentCall {
    let identity = CallIdentity::new(
        limits,
        CallIdentityParts {
            owner: u64::MAX,
            sequence: u32::MAX,
            retry: u32::MAX,
            id: ceiling_text(limits.call_identity_id),
        },
    )
    .expect("whole provider call identity");
    CurrentCall::new(
        limits,
        CurrentCallParts {
            identity,
            tool: ceiling_text(limits.current_call_tool),
            input: ceiling_text(limits.current_call_input),
            cut: true,
            began: None,
        },
    )
    .expect("raw provisional call")
}

pub fn decoded_call(limits: &Limits) -> Call {
    Call::new(
        limits,
        CallParts {
            number: u64::MAX,
            tool: ceiling_text(limits.call_tool),
            input: ceiling_text(limits.call_input),
            outcome: Some(ceiling_text(limits.call_outcome_some)),
            failed: true,
            objects: List::with_capacity(0),
            parts: Box::from([]),
        },
    )
    .expect("decoded source call")
}

pub fn raw_live_call(limits: &Limits) -> LiveCall {
    LiveCall::Raw(LiveCallRaw::new(limits, LiveCallRawParts { value: current_call(limits) }).expect("raw call source"))
}

pub fn decoded_live_call(limits: &Limits) -> LiveCall {
    LiveCall::Decoded(
        LiveCallDecoded::new(limits, LiveCallDecodedParts { value: decoded_call(limits) })
            .expect("decoded call source"),
    )
}

pub fn run_text(limits: &Limits, call: LiveCall) -> RunText {
    let turn = ProvisionalTurnName::new(
        limits,
        ProvisionalTurnNameParts { attempt: u64::MAX, number: u32::MAX, retry: u32::MAX },
    )
    .expect("whole provisional retry name");
    RunText::new(
        limits,
        RunTextParts {
            attempt: u64::MAX,
            began: Some(u64::MAX),
            turn,
            from: u32::MAX,
            through: u32::MAX,
            text: ceiling_text(limits.run_text_text),
            cut: true,
            call: Some(call),
        },
    )
    .expect("provisional text at its configured ceiling")
}

pub fn ended(limits: &Limits, why: WatchEnd) -> Ended {
    Ended::new(limits, EndedParts { watch: u64::MAX, why }).expect("named ending")
}

pub fn stream_events(limits: &Limits) -> [StreamEvent; 5] {
    let delivery = Delivery::new(limits, DeliveryParts { watch: u64::MAX, document: stream_owned(limits) })
        .expect("whole owned delivery");
    [
        StreamEvent::Hello(hello(limits)),
        StreamEvent::Delivery(delivery),
        StreamEvent::Text(stream_text(limits, raw_live_call(limits))),
        StreamEvent::Text(stream_text(limits, decoded_live_call(limits))),
        StreamEvent::Ended(ended(limits, WatchEnd::SubjectEnded)),
    ]
}

pub fn tree_subject(limits: &Limits) -> Subject {
    Subject::Tree(
        SubjectTree::new(limits, SubjectTreeParts { task: u64::MAX, window: u32::MAX }).expect("whole tree window"),
    )
}

pub fn open_tree(limits: &Limits) -> Open {
    let subject = tree_subject(limits);
    let record = encoded(&subject, Subject::measure, Subject::encode);
    let subject = Owned::new(limits, OwnedParts { owner: Owner::Jig, version: VERSION, record })
        .expect("jig subject in its owner's version");
    Open::new(limits, OpenParts { stream: [7; 16], watch: u64::MAX, subject }).expect("open named tree")
}

pub fn deployment_parts(limits: &Limits, row_count: u32) -> DeploymentParts {
    let row = Capacity::new(limits, CapacityParts { project: Some(u32::MAX), live: u32::MAX, limit: u32::MAX })
        .expect("whole capacity row");
    DeploymentParts {
        hosts: Box::from([]),
        accounts: Box::from([]),
        capacity: repeated(row, row_count),
        proposals: u32::MAX,
        refusals: Box::from([]),
        parts: Box::from([]),
    }
}

pub fn waiting_account_state(limits: &Limits, until: Option<u64>) -> OwnerAccountState {
    OwnerAccountState::Later(
        OwnerAccountStateLater::new(limits, OwnerAccountStateLaterParts { until })
            .expect("supplied optional next-ask wall"),
    )
}

pub fn owner_account(limits: &Limits, state: OwnerAccountState) -> OwnerAccount {
    OwnerAccount::new(
        limits,
        OwnerAccountParts {
            provider_name: None,
            manage_usage: None,
            number: u32::MAX,
            place: 0,
            uses: None,
            provider: None,
            display: DocumentText::new(limits, DocumentTextParts { bytes: Box::from([]), cut: false })
                .expect("empty display"),
            holder: OwnerAccountHolder::Deployment,
            sources: Box::from([]),
            generation: Some(u64::MAX),
            state,
            skein_bools: [false, true, false, true, true],
            spent_until: None,
            runs: Box::from([]),
            waiting: Box::from([]),
        },
    )
    .expect("whole supplied account facts")
}

pub fn historical_goal_holding(limits: &Limits) -> OwnerGoalHolding {
    let mut path = List::with_capacity(2);
    path.push(Box::from(b"one/two".as_slice())).expect("first literal segment");
    path.push(Box::from([255, 0])).expect("second opaque segment");
    OwnerGoalHolding::Write(
        OwnerGoalHoldingWrite::new(limits, OwnerGoalHoldingWriteParts { connector: u16::MAX, path, kind: 0 })
            .expect("whole raw goal holding"),
    )
}

pub fn stream_text(limits: &Limits, call: LiveCall) -> StreamText {
    StreamText::new(limits, StreamTextParts { watch: 37, value: run_text(limits, call) })
        .expect("text tagged with its watch")
}
