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
    for (notifications, expected_signed_out, removing) in
        [(&[][..], None, false), (&[true, true][..], Some(true), true), (&[false, false][..], Some(false), true)]
    {
        let mut service = fixtures::started(&fixtures::accounts_document());
        for _ in 0_u32..8 {
            fixtures::turn(&mut service);
        }
        let engine = fixtures::engine(&mut service);
        let mut accounts = engine.accounts.take().expect("accounts runtime");
        let mut events = Queue::with_capacity(16);
        let old = 1;
        let fresh = 9001;
        let candidate = accounts.env.limits.client.document;
        let mut record = record(fresh, &candidate);
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
        let Some(root::Event::Core(core::Event::Account(domain::Event::Component(domain::FromComponent::Failed {
            account: 1,
            failure: domain::Failure::SignInAgain,
        })))) = events.pop()
        else {
            panic!("revocation fails needs sign-in immediately");
        };
        assert!(engine.values.accounts.value(old, 1, Time::from_nanos(1)).is_none());
        let Some(Erasing::Revoked) = accounts.erasing.get(&old) else {
            panic!("erase awaits durability");
        };
        record.key = old;
        let records::Next::Store(before) = accounts.records.keep(old, record.clone(), &candidate) else {
            panic!("the old slot remains admitted");
        };
        assert!(before.replaces.is_some(), "submission did not clear the held digest");
        assert!(accounts.records.stale(old).is_some());
        for confirmed in notifications {
            accounts.above.push(driver::Event::SignedOut { account: old, revoked: *confirmed });
        }
        accounts.iterate(Time::from_nanos(1), Wall::from_nanos(1), &mut engine.values, &mut events, &mut engine.store);
        assert_eq!(engine.store.above.len(), 1, "one erase for revocation and concurrent sign-out");
        assert!(events.is_empty(), "sign-out awaits erasure");
        let terminal = engine.store.above.pop().expect("held erase acknowledgement");
        let store::Event::Erased { .. } = terminal else {
            panic!("held erase acknowledgement");
        };
        engine.store.sources.push(terminal);
        accounts.answers(&mut engine.store, &mut events);
        // A duplicate acknowledgement and an unknown old-source acknowledgement do nothing.
        engine.store.sources.push(store::Event::Erased { owner: Token::new(PREFIX | u64::from(old)) });
        engine.store.sources.push(store::Event::Erased { owner: Token::new(PREFIX | u64::from(8000_u32)) });
        accounts.answers(&mut engine.store, &mut events);
        assert_eq!(signed_out(&mut events, old), expected_signed_out);
        let records::Next::Store(after) = accounts.records.keep(old, record, &candidate) else {
            panic!("erasure retained the slot");
        };
        assert_eq!(after.replaces, None, "only the acknowledgement cleared the digest");
        assert!(accounts.ids.contains_key(&old), "only Removed frees the driver's place");
        assert_eq!(accounts.erasing.contains_key(&old), removing);
        assert_eq!(accounts.above.is_empty(), !removing, "only sign-out asks the driver to remove its account");
        assert!(engine.values.accounts.value(fresh, 2, Time::from_nanos(1)).is_some());
        let Some(store::Event::Secret { value: Some(_), .. }) = engine.store.secrets(
            Token::new(99),
            store::Secrets::Get { owner: store::SecretOwner::Source, number: u64::from(fresh) },
        ) else {
            panic!("the fresh different source keeps its record");
        };
    }
}

fn record(source: u32, limits: &oauth::Limits) -> oauth::SavedToken {
    let mut record = handed_in::record(
        source,
        Secret::new(bytes::copy_of(b"fresh-token")),
        Wall::from_nanos(Duration::from_secs(60).as_nanos()),
        None,
        limits,
    )
    .expect("record fits");
    record.generation = 2;
    record
}
fn signed_out(events: &mut Queue<root::Event>, old: u32) -> Option<bool> {
    let event = events.pop()?;
    let root::Event::Core(core::Event::Account(domain::Event::SignedOut { source, confirmed })) = event else {
        panic!("only a sign-out terminal follows erasure");
    };
    assert_eq!(source, old);
    assert!(events.is_empty(), "one terminal despite duplicate acknowledgements");
    Some(confirmed)
}
