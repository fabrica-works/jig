//! Client v10 account changes and typed outcomes.
use jig_client_protocol_wire::v10::*;
use skein_lib::{List, Reader, Writer};
#[test]
fn all_eight_account_changes_and_their_outcomes_round_trip() {
    let mut uses = List::with_capacity(16);
    for n in 0..16 {
        uses.push(if n % 2 == 0 {
            OwnerAccountUse::Party(
                OwnerAccountUseParty::new(&CEILINGS, OwnerAccountUsePartyParts { party: u64::from(n) })
                    .expect("bounded account wire fixture"),
            )
        } else {
            OwnerAccountUse::Project(
                OwnerAccountUseProject::new(&CEILINGS, OwnerAccountUseProjectParts { project: n })
                    .expect("bounded account wire fixture"),
            )
        })
        .expect("bounded account wire fixture");
    }
    for change in [
        AccountChange::AddKey(
            AccountChangeAddKey::new(
                &CEILINGS,
                AccountChangeAddKeyParts { provider: 3, join: Some(7), digest: [19; 32] },
            )
            .expect("bounded account wire fixture"),
        ),
        AccountChange::ReplaceKey(
            AccountChangeReplaceKey::new(&CEILINGS, AccountChangeReplaceKeyParts { source: 9, digest: [23; 32] })
                .expect("bounded account wire fixture"),
        ),
        AccountChange::RemoveKey(
            AccountChangeRemoveKey::new(&CEILINGS, AccountChangeRemoveKeyParts { source: 9 })
                .expect("bounded account wire fixture"),
        ),
        AccountChange::Resume(
            AccountChangeResume::new(&CEILINGS, AccountChangeResumeParts { source: 9 })
                .expect("bounded account wire fixture"),
        ),
        AccountChange::Remove(
            AccountChangeRemove::new(&CEILINGS, AccountChangeRemoveParts { account: 7 })
                .expect("bounded account wire fixture"),
        ),
        AccountChange::MoveAccount(
            AccountChangeMoveAccount::new(&CEILINGS, AccountChangeMoveAccountParts { account: 7, place: 2 })
                .expect("bounded account wire fixture"),
        ),
        AccountChange::MoveSource(
            AccountChangeMoveSource::new(&CEILINGS, AccountChangeMoveSourceParts { account: 7, source: 9, place: 2 })
                .expect("bounded account wire fixture"),
        ),
        AccountChange::SetUses(
            AccountChangeSetUses::new(&CEILINGS, AccountChangeSetUsesParts { account: 7, uses: Some(uses) })
                .expect("bounded account wire fixture"),
        ),
    ] {
        for holder in [AccountHolder::Own, AccountHolder::Deployment] {
            let ask = Ask::AccountChange(
                AskAccountChange::new(&CEILINGS, AskAccountChangeParts { holder, change: change.clone() })
                    .expect("bounded account wire fixture"),
            );
            let mut out = Writer::new(usize::try_from(ask.measure()).expect("measured ask fits"));
            ask.encode(&mut out).expect("bounded account wire fixture");
            assert_eq!(Ask::decode(&CEILINGS, &mut Reader::new(&out.finish())), Ok(ask));
        }
    }
    for outcome in [
        OwnerOutcome::AccountKeyKept(
            OwnerOutcomeAccountKeyKept::new(&CEILINGS, OwnerOutcomeAccountKeyKeptParts { source: u32::MAX })
                .expect("bounded account wire fixture"),
        ),
        OwnerOutcome::AccountChanged,
        OwnerOutcome::AccountRemoved(
            OwnerOutcomeAccountRemoved::new(&CEILINGS, OwnerOutcomeAccountRemovedParts { confirmed: true })
                .expect("bounded account wire fixture"),
        ),
    ] {
        let mut out = Writer::new(usize::try_from(outcome.measure()).expect("measured outcome fits"));
        outcome.encode(&mut out).expect("bounded account wire fixture");
        assert_eq!(OwnerOutcome::decode(&CEILINGS, &mut Reader::new(&out.finish())), Ok(outcome));
    }
    let mut uses = List::with_capacity(17);
    for _ in 0..17 {
        uses.push(OwnerAccountUse::Party(
            OwnerAccountUseParty::new(&CEILINGS, OwnerAccountUsePartyParts { party: 1 })
                .expect("bounded account wire fixture"),
        ))
        .expect("bounded account wire fixture");
    }
    assert!(AccountChangeSetUses::new(&CEILINGS, AccountChangeSetUsesParts { account: 1, uses: Some(uses) }).is_err());
    let mut out = Writer::new(usize::try_from(Refusal::Provider.measure()).expect("measured refusal fits"));
    Refusal::Provider.encode(&mut out).expect("bounded account wire fixture");
    assert_eq!(Refusal::decode(&CEILINGS, &mut Reader::new(&out.finish())), Ok(Refusal::Provider));
}
