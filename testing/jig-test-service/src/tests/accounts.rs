//! Refusal cleanup through the service's actual handler and routed store answers.
use super::*;
use crate::tests::fixtures;
use skein_lib::{Duration, bytes};

#[test]
fn a_revoked_record_waits_for_erased_and_coalesces_a_sign_out() {
    assert_eq!(
        Map::<u32, Erasing>::worst_case(2),
        Map::<u32, Option<bool>>::worst_case(2),
        "the generalized pending state fits the existing map bound"
    );
    for confirmed in [None, Some(true), Some(false)] {
        let mut service = fixtures::started(&fixtures::accounts_document());
        for _ in 0..8 {
            fixtures::turn(&mut service);
        }
        let engine = fixtures::engine(&mut service);
        let mut accounts = engine.accounts.take().expect("accounts runtime");
        let mut events = Queue::with_capacity(16);
        let old = 1;
        let fresh = 9001;
        let candidate = accounts.env.limits.client.document;
        let mut record = handed_in::record(
            old,
            Secret::new(bytes::copy_of(b"fresh-token")),
            Wall::from_nanos(Duration::from_secs(60).as_nanos()),
            None,
            &candidate,
        )
        .expect("record fits");
        record.generation = 2;
        record.key = fresh;
        let records::Next::Store(keep) = accounts.records.keep(fresh, record.clone(), &candidate) else {
            panic!("fresh source is kept");
        };
        replace(&mut engine.store, keep);
        assert!(accounts.records.kept(fresh).is_some());
        // Hold unrelated store answers apart from this erase's terminal.
        while engine.store.above.pop().is_some() {}
        engine
            .values
            .accounts
            .granted(
                fresh,
                2,
                table::Value { token: Secret::new(bytes::copy_of(b"fresh-token")), scheme: table::Scheme::Bearer },
                Time::from_nanos(Duration::from_secs(60).as_nanos()),
            )
            .expect("fresh value fits");
        accounts.above.push(driver::Event::Failed {
            account: old,
            ends: driver::Ends::Refresh,
            failure: driver::Failure::Exchange(oauth::Failure::Refused),
        });
        accounts.iterate(Time::from_nanos(1), Wall::from_nanos(1), &mut engine.values, &mut events, &mut engine.store);
        assert!(events.iter().any(|event| matches!(
            event,
            root::Event::Core(core::Event::Account(domain::Event::Component(domain::FromComponent::Failed {
                account: 1,
                failure: domain::Failure::SignInAgain
            })))
        )));
        assert!(engine.values.accounts.value(old, 1, Time::from_nanos(1)).is_none());
        assert!(matches!(accounts.erasing.get(&old), Some(Erasing::Revoked)));
        record.key = old;
        let records::Next::Store(before) = accounts.records.keep(old, record.clone(), &candidate) else {
            panic!("the old slot remains admitted");
        };
        assert!(before.replaces.is_some(), "submission did not clear the held digest");
        assert!(accounts.records.stale(old).is_some());
        if let Some(confirmed) = confirmed {
            accounts.above.push(driver::Event::SignedOut { account: old, revoked: confirmed });
            accounts.above.push(driver::Event::SignedOut { account: old, revoked: confirmed });
        }
        accounts.iterate(Time::from_nanos(1), Wall::from_nanos(1), &mut engine.values, &mut events, &mut engine.store);
        assert_eq!(engine.store.above.len(), 1, "one erase for revocation and concurrent sign-out");
        assert!(
            !(&events)
                .into_iter()
                .any(|event| matches!(event, root::Event::Core(core::Event::Account(domain::Event::SignedOut { .. }))))
        );
        let terminal = engine.store.above.pop().expect("held erase acknowledgement");
        assert!(matches!(terminal, store::Event::Erased { .. }));
        engine.store.sources.push(terminal);
        accounts.answers(&mut engine.store, &mut events);
        // A duplicate acknowledgement and an unknown old-source acknowledgement do nothing.
        engine.store.sources.push(store::Event::Erased { owner: Token::new(PREFIX | u64::from(old)) });
        engine.store.sources.push(store::Event::Erased { owner: Token::new(PREFIX | 8000) });
        accounts.answers(&mut engine.store, &mut events);
        let signed_out: Vec<_> = (&events)
            .into_iter()
            .filter_map(|event| match event {
                root::Event::Core(core::Event::Account(domain::Event::SignedOut { source, confirmed })) => {
                    Some((*source, *confirmed))
                }
                _ => None,
            })
            .collect();
        assert_eq!(signed_out, confirmed.map(|confirmed| vec![(old, confirmed)]).unwrap_or_default());
        let records::Next::Store(after) = accounts.records.keep(old, record, &candidate) else {
            panic!("erasure retained the slot");
        };
        assert_eq!(after.replaces, None, "only the acknowledgement cleared the digest");
        if confirmed.is_none() {
            assert!(accounts.ids.contains_key(&old));
            assert!(!accounts.erasing.contains_key(&old));
            assert!(accounts.above.is_empty(), "revocation never asks the driver to remove its account");
        }
        assert!(engine.values.accounts.value(fresh, 2, Time::from_nanos(1)).is_some());
        assert!(matches!(
            engine.store.secrets(
                Token::new(99),
                store::Secrets::Get { owner: store::SecretOwner::Source, number: u64::from(fresh) }
            ),
            Some(store::Event::Secret { value: Some(_), .. })
        ));
    }
}
