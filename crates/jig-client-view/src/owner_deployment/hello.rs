//! Advertised host capabilities and the reason a hello was refused.
use super::{
    Builder, Hole, Key, Owner, OwnerHelloRefusal, OwnerHelloRefusalLine, P, Words, duration, field_number, region, text,
};
pub(super) fn hello(b: &mut Builder, words: &Words, value: &OwnerHelloRefusalLine, position: u64) {
    region(
        b,
        words,
        P::Host,
        &[Hole::Number(value.channel)],
        Some(Key { owner: Owner::Jig.opaque(), kind: 97, a: position, b: value.channel }),
    );
    field_number(b, words, P::Host, value.channel);
    field_number(b, words, P::Slots, u64::from(value.slots));
    text(b, words, P::At, &[Hole::Wall(words.adjusted_wall(value.at))]);
    duration(b, words, P::StopBound, value.stop_bound);
    match value.why {
        OwnerHelloRefusal::Agent { why } => agent(b, words, why),
        OwnerHelloRefusal::ReservedChannel => text(b, words, P::HelloRefusalReservedChannel, &[]),
        OwnerHelloRefusal::Workers { limit } => {
            text(b, words, P::HelloRefusalWorkers, &[]);
            field_number(b, words, P::Limit, u64::from(limit));
        }
        OwnerHelloRefusal::StopBound { grace } => {
            text(b, words, P::HelloRefusalStopBound, &[]);
            duration(b, words, P::AllowedGrace, grace);
        }
    }
    b.close();
}
fn agent(b: &mut Builder, words: &Words, why: jig_client::contract::OwnerAgentShortfall) {
    use crate::owner_deployment_words::AgentPhrase as A;
    use jig_client::contract::OwnerAgentShortfall as S;
    let phrase = match why {
        S::Model { .. } => A::Model,
        S::Window { .. } => A::Window,
        S::Output { .. } => A::Output,
        S::Turns => A::Turns,
        S::Spend => A::Spend,
        S::Time => A::Time,
        S::Waiting => A::Waiting,
        S::Checks => A::Checks,
        S::Message => A::Message,
        S::Inbox => A::Inbox,
        S::Unacknowledged => A::Unacknowledged,
    };
    match why {
        S::Model { charter, model } | S::Window { charter, model } | S::Output { charter, model } => {
            text(b, words, P::Agent(phrase), &[Hole::Number(u64::from(charter))]);
            text(b, words, P::Agent(A::ModelPosition), &[Hole::Number(u64::from(model))]);
        }
        S::Turns | S::Spend | S::Time | S::Waiting | S::Checks | S::Message | S::Inbox | S::Unacknowledged => {
            text(b, words, P::Agent(phrase), &[]);
        }
    }
}
