use super::*;
use skein_lib::List;
pub fn encoded_delivery(value: &w::DeliveryValue) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut writer).unwrap();
    writer.finish()
}
pub fn jig_delivery(value: &w::DeliveryValue, name: u64) -> Box<[u8]> {
    let document = w::Owned::new(
        &w::CEILINGS,
        w::OwnedParts {
            owner: w::Owner::Jig,
            version: jig_protocol_client_wire::VERSION,
            record: encoded_delivery(value),
        },
    )
    .unwrap();
    event_bytes(&w::StreamEvent::Delivery(
        w::Delivery::new(&w::CEILINGS, w::DeliveryParts { watch: name, document }).unwrap(),
    ))
}
pub fn run_event(name: u64) -> Box<[u8]> {
    event_bytes(&w::StreamEvent::Text(
        w::StreamText::new(&limits().wire, w::StreamTextParts { watch: name, value: text_record() }).unwrap(),
    ))
}
pub fn owner_tree_record() -> w::OwnerTree {
    w::OwnerTree::new(
        &limits().wire,
        w::OwnerTreeParts {
            root: chip_record(),
            rows: Box::from([]),
            folded: List::with_capacity(0),
            held_below: 19,
            next: None,
        },
    )
    .unwrap()
}
pub fn owner_tree() -> c::OwnerTree {
    c::OwnerTree {
        root: chip(),
        rows: List::with_capacity(2),
        folded: List::with_capacity(0),
        held_below: 19,
        next: None,
    }
}
pub fn owner_inbox_record() -> w::OwnerInbox {
    w::OwnerInbox::new(
        &limits().wire,
        w::OwnerInboxParts {
            party: 19,
            needs: read_wire_owner_inbox_page().rows().into(),
            updates: Box::from([]),
            next: None,
            cached_needs: 7,
            cached_updates: w::Count::new(&limits().wire, w::CountParts { value: 9, limited: true }).unwrap(),
            results_seen: 13,
            updates_time_complete: false,
            selection: Some(Box::from([])),
        },
    )
    .unwrap()
}
pub fn owner_inbox() -> c::OwnerInbox {
    c::OwnerInbox {
        party: 19,
        needs: read_value_owner_inbox_page().rows,
        updates: List::with_capacity(2),
        next: None,
        cached_needs: 7,
        cached_updates: c::Count { value: 9, limited: true },
        results_seen: 13,
        updates_time_complete: false,
        selection: Some(List::with_capacity(2)),
    }
}
pub fn delivery_cases() -> Box<[(w::DeliveryValue, c::Delivery)]> {
    let mut cases = List::with_capacity(16);
    delivery_cases_first(&mut cases);
    delivery_cases_last(&mut cases);
    cases.into_boxed()
}
fn delivery_cases_first(cases: &mut List<(w::DeliveryValue, c::Delivery)>) {
    cases
        .push((
            w::DeliveryValue::OwnerProject(owner_project_record()),
            c::Delivery::OwnerProject(Box::new(owner_project())),
        ))
        .unwrap();
    cases.push((w::DeliveryValue::Frame(frame_record()), c::Delivery::Frame(Box::new(frame())))).unwrap();
    cases.push((w::DeliveryValue::Task(Box::new(task_record())), c::Delivery::Task(Box::new(task())))).unwrap();
    cases
        .push((
            w::DeliveryValue::OwnerProposal(read_wire_owner_proposal()),
            c::Delivery::OwnerProposal(Box::new(read_value_owner_proposal())),
        ))
        .unwrap();
    cases.push((w::DeliveryValue::Tree(tree_record()), c::Delivery::Tree(Box::new(tree())))).unwrap();
    cases
        .push((w::DeliveryValue::OwnerTree(owner_tree_record()), c::Delivery::OwnerTree(Box::new(owner_tree()))))
        .unwrap();
    cases.push((w::DeliveryValue::Inbox(inbox_record()), c::Delivery::Inbox(Box::new(inbox())))).unwrap();
    cases
        .push((w::DeliveryValue::OwnerInbox(owner_inbox_record()), c::Delivery::OwnerInbox(Box::new(owner_inbox()))))
        .unwrap();
}
fn delivery_cases_last(cases: &mut List<(w::DeliveryValue, c::Delivery)>) {
    cases.push((w::DeliveryValue::Goals(goals_record()), c::Delivery::Goals(Box::new(goals())))).unwrap();
    cases.push((w::DeliveryValue::Tasks(tasks_record()), c::Delivery::Tasks(Box::new(tasks())))).unwrap();
    cases
        .push((
            w::DeliveryValue::Conversations(conversations_record()),
            c::Delivery::Conversations(Box::new(conversations())),
        ))
        .unwrap();
    cases.push((w::DeliveryValue::Run(text_record()), c::Delivery::Run(Box::new(run_text_expected())))).unwrap();
    cases.push((w::DeliveryValue::RunState(run_state_record()), c::Delivery::RunState(Box::new(run_state())))).unwrap();
    cases
        .push((w::DeliveryValue::Deployment(deployment_record()), c::Delivery::Deployment(Box::new(deployment()))))
        .unwrap();
    cases
        .push((
            w::DeliveryValue::OwnerDeployment(owner_deployment_record()),
            c::Delivery::OwnerDeployment(Box::new(owner_deployment())),
        ))
        .unwrap();
    let value = w::Chips::new(&limits().wire, w::ChipsParts { rows: chip_pieces(1) }).unwrap();
    let mut chips = List::with_capacity(2);
    chips.push(chip()).unwrap();
    cases.push((w::DeliveryValue::Chips(value), c::Delivery::Chips(chips))).unwrap();
}
impl Harness {
    pub fn delivered(&self) -> Option<&c::Delivery> {
        for request in &self.out {
            match request {
                Request::Watch { watch, event: c::WatchEvent::Delivered(value) } if *watch == WATCH => {
                    return Some(value);
                }
                Request::Exchange { .. }
                | Request::Abandon { .. }
                | Request::Answer { .. }
                | Request::Read { .. }
                | Request::OpenStream
                | Request::CloseStream
                | Request::Watch { .. }
                | Request::FamilyDelivery { .. }
                | Request::Ended { .. }
                | Request::Hello { .. }
                | Request::Fact { .. } => {}
            }
        }
        None
    }
}
pub fn inbox_row() -> w::OwnerInboxEntry {
    let lim = limits().wire;
    let cursor = inbox_cursor();
    let task = w::OwnerTaskLink::Stored(
        w::OwnerTaskLinkStored::new(&lim, w::OwnerTaskLinkStoredParts { number: 41, status: None }).unwrap(),
    );
    w::OwnerInboxEntry::new(
        &lim,
        w::OwnerInboxEntryParts {
            reply: None,
            task,
            project: Some(7),
            object: w::OwnerInboxObject::PersonTask,
            activity: Some(19),
            at: Some(u64::MAX),
            unread: Some(false),
            cursor,
        },
    )
    .unwrap()
}
pub fn inbox_cursor() -> w::OwnerInboxCursor {
    w::OwnerInboxCursor::Number(
        w::OwnerInboxCursorNumber::new(&limits().wire, w::OwnerInboxCursorNumberParts { number: u64::MAX }).unwrap(),
    )
}
pub fn inbox_row_expected() -> c::OwnerInboxEntry {
    c::OwnerInboxEntry {
        reply: None,
        task: c::OwnerTaskLink::Stored { number: 41, status: None },
        project: Some(7),
        object: c::OwnerInboxObject::PersonTask,
        activity: Some(19),
        at: Some(skein_lib::Wall::from_nanos(u64::MAX)),
        unread: Some(false),
        cursor: c::OwnerInboxCursor::Number { number: u64::MAX },
    }
}
pub fn inbox_pieces(count: u32) -> Box<[u8]> {
    let row = inbox_row();
    let mut encoded = Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut encoded).unwrap();
    let encoded = encoded.finish();
    let mut piece =
        Writer::new(encoded.len().checked_add(4).unwrap().checked_mul(usize::try_from(count).unwrap()).unwrap());
    for _ in 0..count {
        skein_codec::piece::write(&mut piece, &encoded).unwrap();
    }
    piece.finish()
}
pub fn owner_inbox_with_rows(count: u32) -> w::DeliveryValue {
    let mut parts = owner_inbox_record().into_parts();
    parts.needs = inbox_pieces(count);
    parts.updates = inbox_pieces(count);
    parts.next = Some(inbox_cursor());
    w::DeliveryValue::OwnerInbox(w::OwnerInbox::new(&limits().wire, parts).unwrap())
}
pub fn owner_inbox_with_rows_expected() -> c::Delivery {
    let mut value = owner_inbox();
    value.needs = List::with_capacity(2);
    value.needs.push(inbox_row_expected()).unwrap();
    value.updates.push(inbox_row_expected()).unwrap();
    value.next = Some(c::OwnerInboxCursor::Number { number: u64::MAX });
    c::Delivery::OwnerInbox(Box::new(value))
}
pub fn tree_row() -> w::OwnerTreeRow {
    w::OwnerTreeRow::new(
        &limits().wire,
        w::OwnerTreeRowParts {
            chip: chip_record(),
            parent: 41,
            depth: 2,
            proposals: List::with_capacity(0),
            spent: 19,
            dependencies: Box::from([]),
            cut: true,
        },
    )
    .unwrap()
}
pub fn tree_pieces(count: u32) -> Box<[u8]> {
    let row = tree_row();
    let mut encoded = Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut encoded).unwrap();
    let encoded = encoded.finish();
    let mut piece =
        Writer::new(encoded.len().checked_add(4).unwrap().checked_mul(usize::try_from(count).unwrap()).unwrap());
    for _ in 0..count {
        skein_codec::piece::write(&mut piece, &encoded).unwrap();
    }
    piece.finish()
}
pub fn owner_tree_with_rows(count: u32) -> w::DeliveryValue {
    let mut parts = owner_tree_record().into_parts();
    parts.rows = tree_pieces(count);
    w::DeliveryValue::OwnerTree(w::OwnerTree::new(&limits().wire, parts).unwrap())
}
pub fn owner_tree_with_rows_expected() -> c::Delivery {
    let mut value = owner_tree();
    value
        .rows
        .push(c::OwnerTreeRow {
            chip: chip(),
            parent: 41,
            depth: 2,
            proposals: List::with_capacity(0),
            spent: 19,
            dependencies: List::with_capacity(2),
            cut: true,
        })
        .unwrap();
    c::Delivery::OwnerTree(Box::new(value))
}
pub fn malformed_delivery_pieces() -> Box<[w::DeliveryValue]> {
    let mut values = List::with_capacity(3);
    let mut inbox = owner_inbox_record().into_parts();
    inbox.needs = Box::from([0, 0, 0, 1]);
    values.push(w::DeliveryValue::OwnerInbox(w::OwnerInbox::new(&limits().wire, inbox).unwrap())).unwrap();
    let mut tree = owner_tree_record().into_parts();
    tree.rows = Box::from([0, 0, 0, 1]);
    values.push(w::DeliveryValue::OwnerTree(w::OwnerTree::new(&limits().wire, tree).unwrap())).unwrap();
    values
        .push(w::DeliveryValue::Chips(
            w::Chips::new(&limits().wire, w::ChipsParts { rows: Box::from([0, 0, 0, 1]) }).unwrap(),
        ))
        .unwrap();
    values.into_boxed()
}
