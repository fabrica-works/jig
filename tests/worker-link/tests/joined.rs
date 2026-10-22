#[path = "../src/joined.rs"]
pub mod joined;
mod joined_story {
    use super::joined::{Joined, application, credential, received, sent, values};
    use jig_domain_host as host;
    use jig_protocol_engine_workers as engine;
    use jig_protocol_worker_link::{Credential, Request};
    use jig_worker_link_world::fixtures::{self, ATTEMPT_TWO, RUN_SEVEN};
    use skein_channel::Closed;
    use skein_lib::{Duration, Secret, Time, Token};
    fn world(seed: u64) -> Joined {
        let mut world = Joined::new(&fixtures::joined_limits(), seed);
        world.run();
        world
    }
    fn assign(world: &mut Joined) {
        world.engine_down(
            host::LinkDown::Assign { assignment: fixtures::assignment() },
            fixtures::assignment_parts(),
            values(),
        );
        world.run();
    }
    fn downs(world: &Joined) -> Vec<&host::LinkDown> {
        world
            .worker_root
            .iter()
            .filter_map(|request| if let Request::Down { down, .. } = request { Some(down) } else { None })
            .collect()
    }
    fn records(world: &Joined) -> Vec<Vec<u16>> {
        world.frames().iter().map(|frames| frames.iter().copied().filter(|kind| *kind >= 256).collect()).collect()
    }
    fn value(request: &Request, expected: &engine::GrantValue) {
        let Request::Value { account, generation, expires, value } = request else { panic!("a layer value") };
        assert_eq!((*account, *generation, *expires), (1, 3, Time::ZERO.saturating_add(Duration::from_secs(60))));
        assert_eq!(received(value), sent(expected));
    }
    #[test]
    fn a_worker_is_admitted_and_its_hello_reaches_the_engine_root() {
        let mut world = world(1);
        let engine::Request::Hello { channel, hello, worker: _ } = &world.engine_root[0] else { panic!("hello") };
        assert_eq!((*channel, hello), (Token::new(1), &fixtures::hello()));
        assert_eq!(world.dials, [Time::ZERO]);
        world.finish();
    }
    #[test]
    fn an_assignments_grant_values_reach_the_worker_layer() {
        let mut world = world(1);
        assign(&mut world);
        assert_eq!(world.worker_order, ["value", "value", "down"]);
        assert_eq!(world.values.len(), 2);
        value(&world.values[0], &credential(engine::Scheme::Bearer));
        value(&world.values[1], &application());
        let host::LinkDown::Assign { assignment } = downs(&world)[0] else { panic!("assignment") };
        assert_eq!(assignment.assignment.grants, fixtures::assignment().assignment.grants);
        world.finish();
    }
    #[test]
    fn a_turn_crosses_and_is_kept_after_a_cut_and_a_redial() {
        for seed in 0..8 {
            let mut world = world(seed);
            assign(&mut world);
            world.acknowledge(false);
            world.worker_up(fixtures::turn(), Box::new([]));
            world.run();
            world.cut();
            world.acknowledge(true);
            world.advance(Duration::from_millis(200));
            assert_eq!(world.engine_turns(), [1]);
            assert_eq!(world.engine_channels(), [Token::new(1), Token::new(2)]);
            let turns = world
                .engine_root
                .iter()
                .filter(|request| matches!(request, engine::Request::Up { up: host::LinkUp::Turn { .. }, .. }))
                .count();
            assert_eq!(turns, 2, "the unacknowledged turn crossed both channels");
            assert_eq!(
                downs(&world).last(),
                Some(&&host::LinkDown::TurnKept { run: RUN_SEVEN, attempt: ATTEMPT_TWO, turn: 1 })
            );
            assert_eq!(records(&world), [vec![0x0100, 0x0101], vec![0x0100, 0x0101]]);
            world.finish();
        }
    }
    #[test]
    fn a_relayed_call_is_answered_across_the_two_sides() {
        let mut world = world(1);
        world.worker_up(fixtures::call(&[0, 2, 7]), Box::new([]));
        world.run();
        assert_eq!(downs(&world), [&fixtures::relay_answer(&[0, 2, 7])]);
        world.finish();
    }
    #[test]
    fn relays_go_again_after_the_hello_before_the_hubs_retained_turns() {
        let mut world = world(3);
        world.acknowledge(false);
        world.worker_up(fixtures::call(&[0, 2, 7]), Box::new([]));
        world.worker_up(fixtures::turn(), Box::new([]));
        world.run();
        world.cut();
        world.acknowledge(true);
        world.advance(Duration::from_millis(200));
        assert_eq!(
            downs(&world),
            [
                &fixtures::relay_answer(&[0, 2, 7]),
                &host::LinkDown::TurnKept { run: RUN_SEVEN, attempt: ATTEMPT_TWO, turn: 1 },
            ]
        );
        assert_eq!(records(&world), [vec![0x0100, 0x0102, 0x0101], vec![0x0100, 0x0102, 0x0101]]);
        world.finish();
    }
    #[test]
    fn an_answer_is_acknowledged_once() {
        let mut world = world(1);
        world.worker_up(fixtures::answer(), fixtures::work_parts());
        world.run();
        assert_eq!(downs(&world), [&host::LinkDown::Acknowledge { run: RUN_SEVEN, attempt: ATTEMPT_TWO }]);
        world.finish();
    }
    fn refusal(credential: Credential, reason: u16) {
        let mut world = Joined::with_credential(&fixtures::joined_limits(), 1, credential);
        world.run();
        assert!(world.engine_root.is_empty());
        assert!(world.worker_root.iter().any(|request| matches!(request,
            Request::Fact { why: Closed::RefusedPeer(found) } if *found == reason)));
        world.advance(Duration::from_millis(200));
        assert!(world.dials.len() >= 2);
        assert!(world.dials[1] >= Time::ZERO.saturating_add(Duration::from_millis(50)));
        assert!(world.dials[1] <= Time::ZERO.saturating_add(Duration::from_millis(100)));
        world.finish();
    }
    #[test]
    fn a_wrong_secret_worker_redials_after_unauthorized() {
        refusal(Credential { secret: Secret::new(b"wrong".as_slice().into()), ..fixtures::credential() }, 256);
    }
    #[test]
    fn another_payload_version_worker_redials_after_application() {
        refusal(Credential { payload: 4, ..fixtures::credential() }, 257);
    }
}
