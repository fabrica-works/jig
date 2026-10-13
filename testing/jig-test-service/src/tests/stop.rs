use super::fixtures;
use crate::{End, stop};
use jig_test_domain as root;

fn components() -> crate::Service {
    {
        let mut service = fixtures::started(&fixtures::workers_document());
        for _ in 0..64 {
            fixtures::turn(&mut service);
            let engine = fixtures::engine(&mut service);
            while let Some(request) = engine.store_requests().pop() {
                match request {
                    crate::StoreRequest::Restart(step) => {
                        engine.store_answers().push(crate::StoreAnswer::Restored { step, rows: Box::default() });
                    }
                    crate::StoreRequest::Commit { number, .. } => {
                        engine.store_answers().push(crate::StoreAnswer::Committed { number, fill: 0 });
                    }
                    crate::StoreRequest::Load(_) | crate::StoreRequest::Read(_) => panic!("only startup records"),
                }
            }
            if !service.work_pending(skein_lib::Time::from_nanos(1)) && fixtures::engine(&mut service).domain.ready() {
                break;
            }
        }
        assert_eq!(service.end(), None);
        assert!(fixtures::engine(&mut service).domain.ready());
        service
    }
}

fn settle(service: &mut crate::Service) {
    for _ in 0_u32..16 {
        fixtures::turn(service);
    }
}

#[test]
fn a_shutdown_stops_the_listeners_before_the_core_hears_stopping() {
    let mut service = components();
    service.shutdown();
    let engine = fixtures::engine(&mut service);
    assert!(!engine.domain.stopping());
    assert_eq!(engine.drain.as_ref().expect("drain").phase, stop::Phase::Stopping);
    assert_eq!(engine.workers.as_ref().expect("workers").ending, None);
    assert_eq!(engine.events.len(), 1);
    let Some(root::Event::Stop) = engine.events.pop() else {
        panic!("the core hears stopping next");
    };
}

#[test]
fn components_close_only_after_the_core_says_drained() {
    let mut service = components();
    service.shutdown();
    let engine = fixtures::engine(&mut service);
    assert_eq!(engine.workers.as_ref().expect("workers").ending, None);
    assert_eq!(engine.drain.as_ref().expect("drain").phase, stop::Phase::Stopping);
    stop::drained(engine);
    assert_eq!(engine.workers.as_ref().expect("workers").ending, Some(stop::Closing::Drain));
    assert_eq!(engine.drain.as_ref().expect("drain").phase, stop::Phase::Closing { waiting: 2 });
}

#[test]
fn the_store_closes_last() {
    let mut service = components();
    service.shutdown();
    stop::drained(fixtures::engine(&mut service));
    assert_eq!(service.end(), None, "components have not reported closed");
    settle(&mut service);
    assert_eq!(service.end(), Some(&End::Drained));
    assert!(fixtures::engine(&mut service).accounts.as_ref().expect("accounts").is_empty());
    assert!(fixtures::engine(&mut service).workers.as_ref().expect("workers").closed());
}

#[test]
fn a_second_shutdown_aborts_each_component() {
    let mut service = components();
    service.shutdown();
    service.shutdown();
    fixtures::turn(&mut service);
    assert_eq!(service.end(), Some(&End::Aborted));
    settle(&mut service);
    assert!(fixtures::engine(&mut service).accounts.as_ref().expect("accounts").is_empty());
    assert!(fixtures::engine(&mut service).workers.as_ref().expect("workers").closed());
}

#[test]
fn an_in_process_workers_listener_sets_workers_in_process() {
    let config = fixtures::config();
    assert!(jig_protocol_config::core(&config, [7; 16], 7).workers_in_process);
}

#[test]
fn a_connector_input_returned_busy_is_kept_until_the_drain_ends() {
    let mut service = components();
    service.shutdown();
    crate::route::request(
        fixtures::engine(&mut service),
        root::Request::Busy { event: Box::new(root::Event::ConnectorTimer { number: 1 }) },
    );
    assert_eq!(fixtures::engine(&mut service).drain.as_ref().expect("drain").busy.len(), 1);
    stop::drained(fixtures::engine(&mut service));
    settle(&mut service);
    assert_eq!(service.end(), Some(&End::Drained));
    assert!(fixtures::engine(&mut service).drain.as_ref().expect("drain").busy.is_empty());
}

#[test]
fn a_call_the_root_answers_busy_while_stopping_reaches_its_worker() {
    let mut service = components();
    service.shutdown();
    crate::route::request(
        fixtures::engine(&mut service),
        root::Request::Deliver(root::Delivery::CallBusy {
            channel: skein_lib::Token::new(17),
            task: 2,
            attempt: 3,
            name: b"call".as_slice().into(),
        }),
    );
    assert_eq!(
        fixtures::engine(&mut service).workers.as_ref().expect("workers").component.dropped_down(),
        1,
        "the busy relay reaches the workers component even when its channel has gone"
    );
    assert_eq!(service.end(), None);
}
