pub fn bounds() -> jig_domain_fleet::Bounds {
    use skein_lib::Duration;
    jig_domain_fleet::Bounds {
        models: Box::new([jig_domain_fleet::ModelBound {
            name: Box::from(&b"scripted"[..]),
            window: 8192,
            output: 1024,
        }]),
        turns: u32::MAX,
        spend: u64::MAX,
        time: Duration::from_secs(3600),
        waiting: Duration::from_secs(3600),
        checks: Duration::from_nanos(1),
        message_bytes: u32::MAX,
        inbox: u32::MAX,
        unacknowledged: u32::MAX,
    }
}

#[must_use]
pub fn needs() -> jig_domain_fleet::Needs {
    jig_domain_fleet::Needs { charters: Box::new([]), spend: 0, message_bytes: 0, start_messages: 0, unacknowledged: 0 }
}

/// Make one bounded batch and apply its projection saves to the unit core.
pub(crate) fn make_batch(
    core: &mut super::Core,
    limits: &super::Limits,
    creator: super::tasks::Party,
    batch: Box<[super::tasks::New]>,
) {
    make_batch_at(core, limits, creator, batch, super::Wall::EPOCH);
}

pub(crate) fn make_batch_at(
    core: &mut super::Core,
    limits: &super::Limits,
    creator: super::tasks::Party,
    batch: Box<[super::tasks::New]>,
    wall: super::Wall,
) {
    use super::{Env, Queue, ReplyTo, Time, Token, projections, room_max, tasks};
    let mut out = Queue::with_capacity(tasks::max_out(&limits.tasks));
    tasks::step(
        &mut core.tasks,
        &Env { now: Time::ZERO, wall, limits: limits.tasks },
        tasks::Event::Make { reply_to: ReplyTo::new(Token::new(1)), creator, batch },
        &mut out,
    );
    while let Some(request) = out.pop() {
        if let tasks::Request::Save { record: tasks::Stored::Live(row) } = request {
            let mut projections = Queue::with_capacity(room_max(limits).expect("projection room").writes);
            projections::save(core, limits, &row, &mut projections);
        }
    }
}
