//! Attempt fencing through the hub's public step boundary (hosts.md, 6.5).
use super::*;
use crate::{LiveAsk, LiveKind, Piece};

fn piece() -> Piece {
    Piece { turn: 1, retry: 3, kind: LiveKind::Text, offset: 7, bytes: Box::from(&b"whole"[..]) }
}

fn assert_activity(record: LinkUp) {
    match record {
        LinkUp::Activity { .. } => {}
        other @ (LinkUp::Piece { .. }
        | LinkUp::Logins { .. }
        | LinkUp::SignedOut { .. }
        | LinkUp::Unauthorized { .. }
        | LinkUp::Limited { .. }
        | LinkUp::AccountRefused { .. }
        | LinkUp::Turn { .. }
        | LinkUp::Call { .. }
        | LinkUp::Answer { .. }) => panic!("only activity leaves: {other:?}"),
    }
}

fn start(h: &mut Harness, attempt: u64) -> Token {
    let mut assignment = assignment(1);
    assignment.attempt = Token::new(attempt);
    assignment.workspace = None;
    assignment.save = false;
    assignment.live = LiveAsk { text: true, ..LiveAsk::NONE };
    let requests = h.step(fixtures::assign(assignment));
    let [Request::Start { owner, .. }] = &*requests else { panic!("itemless start") };
    let owner = *owner;
    assert!(h.step(Event::Started { owner, agent: Token::new(attempt) }).is_empty());
    owner
}

#[test]
fn an_old_agents_piece_and_its_queued_piece_are_fenced_after_reassignment() {
    let limits = Limits { slots: 1, told: 8, ..LIMITS };
    let mut h = Harness {
        domain: Domain::new(&limits),
        env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        out: Queue::with_capacity(max_out(&limits)),
    };
    assert_eq!((h.domain.pieces_fenced(), h.domain.pieces_lost()), (0, 0));
    let old = start(&mut h, 1001);
    while h.domain.pop_activity().is_some() {}
    assert!(h.step(Event::Piece { owner: old, piece: piece() }).is_empty());
    assert_eq!((h.domain.pieces_fenced(), h.domain.pieces_lost()), (0, 0));
    assert_eq!(
        &*h.step(fixtures::finished(old, Finish::Ended { outcome: Box::new([]) })),
        [Request::Stop { agent: Token::new(1001) }]
    );
    let requests = h.step(Event::Gone { owner: old, detail: Box::new([]) });
    let [Request::Up(LinkUp::Answer { .. })] = &*requests else { panic!("the old attempt answered") };
    assert!(h.step(Event::Down(LinkDown::Acknowledge { run: Token::new(1), attempt: Token::new(1001) })).is_empty());
    h.domain.reclaim();
    let current = start(&mut h, 1002);
    assert_ne!(old, current, "the reclaimed slot has a new generation");
    assert!(h.step(Event::Piece { owner: old, piece: piece() }).is_empty());
    assert_eq!((h.domain.pieces_fenced(), h.domain.pieces_lost()), (1, 0));
    let mut activity = 0_u32;
    while let Some(record) = h.domain.pop_activity() {
        assert_activity(record);
        activity += 1;
    }
    assert_eq!(activity, 5, "ending and answered phases, then the new preparing, starting and active phases");
    assert_eq!((h.domain.pieces_fenced(), h.domain.pieces_lost()), (2, 0));
    assert!(h.step(Event::Piece { owner: current, piece: piece() }).is_empty());
    assert_eq!(
        h.domain.pop_activity(),
        Some(LinkUp::Piece { run: Token::new(1), attempt: Token::new(1002), piece: piece() })
    );
    assert_eq!(h.domain.pop_activity(), None);
    assert_eq!((h.domain.pieces_fenced(), h.domain.pieces_lost()), (2, 0));
}

#[test]
fn pieces_are_fenced_before_the_agent_is_live_and_after_it_leaves_live() {
    let mut h = Harness::new();
    assert_eq!((h.domain.pieces_fenced(), h.domain.pieces_lost()), (0, 0));
    let mut assignment = assignment(1);
    assignment.save = false;
    assignment.live = LiveAsk { text: true, ..LiveAsk::NONE };
    let requests = h.step(fixtures::assign(assignment));
    let [Request::Prepare { owner, .. }] = &*requests else { panic!("prepare") };
    let owner = *owner;
    assert!(h.step(Event::Piece { owner, piece: piece() }).is_empty());
    assert_eq!((h.domain.pieces_fenced(), h.domain.pieces_lost()), (1, 0));
    h.step(Event::Prepared { owner, workspace: Token::new(201) });
    assert!(h.step(Event::Piece { owner, piece: piece() }).is_empty());
    assert_eq!((h.domain.pieces_fenced(), h.domain.pieces_lost()), (2, 0));
    assert!(h.step(Event::Started { owner, agent: Token::new(301) }).is_empty());
    h.step(fixtures::finished(owner, Finish::Ended { outcome: Box::new([]) }));
    assert!(h.step(Event::Piece { owner, piece: piece() }).is_empty());
    assert_eq!((h.domain.pieces_fenced(), h.domain.pieces_lost()), (3, 0));
    h.step(Event::Gone { owner, detail: Box::new([]) });
    assert!(h.step(Event::Piece { owner, piece: piece() }).is_empty());
    while let Some(record) = h.domain.pop_activity() {
        assert_activity(record);
    }
    assert_eq!((h.domain.pieces_fenced(), h.domain.pieces_lost()), (4, 0));
}
