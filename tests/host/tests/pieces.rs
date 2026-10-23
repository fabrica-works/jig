//! Live pieces through the existing scripted agent and engine peers (hosts.md, 6.5 and 11).
use jig_domain_host::{FromAgent, LinkUp, LiveAsk, LiveKind, Phase, Piece};
use jig_host_world::turn_world::World;
use skein_lib::Token;

const TEXT: LiveAsk = LiveAsk { text: true, ..LiveAsk::NONE };

fn world(told: u32, piece_bytes: u32, live: LiveAsk) -> World {
    let mut world = World::with_piece_limits(told, piece_bytes);
    assert_eq!(world.piece_counts(), (0, 0));
    world.assign_live_conversation(Box::new([]), Box::new([]), live);
    assert_eq!(world.started_state().expect("agent started").live, live);
    world
}

fn text(turn: u32, retry: u32, offset: u64, bytes: &[u8]) -> Piece {
    Piece { turn, retry, kind: LiveKind::Text, offset, bytes: Box::from(bytes) }
}

fn pieces(world: &World) -> Vec<(Token, Token, Piece)> {
    world
        .reports
        .iter()
        .filter_map(|record| match record {
            LinkUp::Piece { run, attempt, piece } => Some((*run, *attempt, piece.clone())),
            LinkUp::Logins { .. }
            | LinkUp::SignedOut { .. }
            | LinkUp::Activity { .. }
            | LinkUp::Unauthorized { .. }
            | LinkUp::Limited { .. }
            | LinkUp::AccountRefused { .. }
            | LinkUp::Turn { .. }
            | LinkUp::Call { .. }
            | LinkUp::Answer { .. } => None,
        })
        .collect()
}

fn expected(piece: Piece) -> (Token, Token, Piece) {
    (Token::new(31), Token::new(932), piece)
}

fn finish(world: &mut World, turns: u32) {
    for number in 1..=turns {
        world.commit_turn(number);
    }
    world.finish(turns, false);
    world.acknowledge_answer();
    world.assert_settled();
}

#[test]
fn three_pieces_precede_their_turn_and_late_text_is_fenced() {
    let mut world = world(2, 16, TEXT);
    let first = [text(1, 1, 0, b"hello"), text(1, 1, 5, b" world"), text(1, 2, 0, b"retry")];
    for piece in &first {
        world.piece(piece.clone());
    }
    assert_eq!(pieces(&world), first.map(expected));
    world.turn(1, b"whole turn");
    assert_eq!(world.link_words, ["piece", "piece", "piece", "turn"]);
    world.piece(text(1, 3, 0, b"late"));
    let next = text(2, 1, u64::from(u32::MAX) + 1, b"next");
    world.piece(next.clone());
    assert_eq!(pieces(&world).last(), Some(&expected(next)));
    assert_eq!(pieces(&world).len(), 4);
    assert_eq!(world.piece_counts(), (1, 0));
    finish(&mut world, 1);
    assert_eq!(world.piece_counts(), (1, 0));
}

#[test]
fn a_queued_piece_is_fenced_after_its_turn_and_the_next_item_still_leaves() {
    let mut world = world(8, 16, TEXT);
    world.lose_contact();
    world.piece(text(1, 1, 0, b"queued"));
    assert_eq!(world.piece_counts(), (0, 0));
    world.turn(1, b"whole turn");
    let next = text(2, 1, 0, b"next");
    world.piece(next.clone());
    world.reconnect();
    assert_eq!(pieces(&world), [expected(next)]);
    assert_eq!(world.piece_counts(), (1, 0));
    finish(&mut world, 1);
    assert_eq!(world.piece_counts(), (1, 0));
}

#[test]
fn an_unasked_kind_and_turn_zero_are_fenced() {
    let input = LiveKind::Input { call: Box::from(&b"provider-id"[..]), tool: Box::from(&b"inspect"[..]) };
    for (live, kinds) in [
        (LiveAsk::NONE, vec![LiveKind::Text, LiveKind::Reasoning, input.clone()]),
        (TEXT, vec![LiveKind::Reasoning, input]),
    ] {
        let mut world = world(2, 16, live);
        let count = u64::try_from(kinds.len()).expect("tiny case");
        for kind in kinds {
            world.piece(Piece { kind, ..text(1, 1, 0, b"unasked") });
        }
        world.piece(text(0, 1, 0, b"no turn zero"));
        assert!(pieces(&world).is_empty());
        assert_eq!(world.piece_counts(), (count + 1, 0));
        finish(&mut world, 0);
        assert_eq!(world.piece_counts(), (count + 1, 0));
    }
}

#[test]
fn each_asked_kind_passes_with_all_fields_whole_while_waiting() {
    for (live, kind) in [
        (TEXT, LiveKind::Text),
        (LiveAsk { reasoning: true, ..LiveAsk::NONE }, LiveKind::Reasoning),
        (
            LiveAsk { input: true, ..LiveAsk::NONE },
            LiveKind::Input { call: vec![b'c'; 256].into_boxed_slice(), tool: vec![b't'; 64].into_boxed_slice() },
        ),
    ] {
        let mut world = world(2, 16, live);
        world.agent_report(FromAgent::Yielded { owner: world.owner() });
        let piece = Piece { kind, ..text(1, 7, u64::MAX, b"whole") };
        world.piece(piece.clone());
        assert_eq!(pieces(&world), [expected(piece)]);
        assert_eq!(world.piece_counts(), (0, 0));
        finish(&mut world, 0);
        assert_eq!(world.piece_counts(), (0, 0));
    }
}

#[test]
fn a_piece_at_the_byte_bound_passes_and_one_byte_more_is_fenced() {
    for limit in [0, 16] {
        let mut world = world(2, limit, TEXT);
        world.piece(text(1, 1, 0, &vec![b'x'; usize::try_from(limit + 1).expect("tiny bound")]));
        let exact = text(1, 2, 0, &vec![b'x'; usize::try_from(limit).expect("tiny bound")]);
        world.piece(exact.clone());
        assert_eq!(pieces(&world), [expected(exact)]);
        assert_eq!(world.piece_counts(), (1, 0));
        finish(&mut world, 0);
        assert_eq!(world.piece_counts(), (1, 0));
    }
}

#[test]
fn pieces_stop_at_half_the_queue_and_leave_room_for_a_phase_change() {
    for (capacity, activity_first) in [(2, false), (5, false), (64, false), (256, false), (2, true), (5, true)] {
        let mut world = world(capacity, 16, TEXT);
        world.lose_contact();
        let room = capacity / 2;
        for offset in 0..room {
            if activity_first {
                world.activity(jig_domain_host::Activity::LongDone);
            } else {
                world.piece(text(1, 1, u64::from(offset), b"x"));
            }
        }
        world.piece(text(1, 1, u64::from(room), b"lost"));
        world.agent_report(FromAgent::Yielded { owner: world.owner() });
        assert_eq!(world.piece_counts(), (0, 1));
        assert_eq!(world.lost_facts(), 0);
        world.reconnect();
        let expected: Vec<_> = if activity_first {
            Vec::new()
        } else {
            (0..room).map(|offset| expected(text(1, 1, u64::from(offset), b"x"))).collect()
        };
        assert_eq!(pieces(&world), expected);
        assert!(matches!(world.reports.last(), Some(LinkUp::Activity {
            run, attempt, activity: jig_domain_host::Activity::Phase(Phase::Waiting),
        }) if (*run, *attempt) == (Token::new(31), Token::new(932))));
        finish(&mut world, 0);
        assert_eq!(world.piece_counts(), (0, 1));
        assert_eq!(world.lost_facts(), 0);
    }
}

#[test]
fn a_queued_piece_is_fenced_when_its_attempt_stops_before_drain() {
    let mut world = world(8, 16, TEXT);
    world.lose_contact();
    world.piece(text(1, 1, 0, b"queued"));
    world.finish(0, false);
    assert_eq!(world.piece_counts(), (0, 0));
    world.reconnect();
    assert!(pieces(&world).is_empty());
    assert_eq!(world.piece_counts(), (1, 0));
    world.acknowledge_answer();
    world.assert_settled();
    assert_eq!(world.piece_counts(), (1, 0));
}
