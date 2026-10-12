use super::revisions::{Harness, task};
use crate::{Event, MessageKind, Party, Request};
use alloc::boxed::Box;
use skein_lib::Wall;

fn refused_news(number: u64) -> crate::Word {
    crate::Word {
        number,
        from: Party::Task(1),
        kind: MessageKind::News { subscription: 10, class: crate::NewsClass::Wakes },
        words: Box::new([]),
        at: Wall::EPOCH,
        hits: 1,
        eligible: false,
    }
}

#[test]
fn merging_refused_news_retires_its_refusal_and_relays_the_new_hint() {
    let mut h = Harness::new();
    h.make([task(1, &[])].into());
    let row = &mut crate::domain::task_mut(&mut h.domain, 1).expect("task").record;
    row.attempt = 1;
    row.phase = crate::Phase::Active(crate::Active::Claimed { attempt: 1 });
    row.last_message = 1;
    row.inbox = Box::new([refused_news(1)]);
    row.refused = Box::new([1]);
    row.subscriptions = Box::new([crate::Subscription {
        number: 10,
        kind: crate::SubscriptionKind::Topic { connector: 0, topic: 10 },
    }]);
    let out = h.send(Event::Notice { task: 1, word: refused_news(2) });
    assert!(h.domain.task(1).expect("task").refused.is_empty());
    let mut relayed = false;
    for request in &out {
        if let Request::Relay { word, .. } = request {
            relayed |= word.number == 2;
        }
    }
    assert!(relayed);
}
