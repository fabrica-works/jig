//! Client v10 account changes and typed outcomes.
use jig_client_protocol_wire::v10::*;
use skein_lib::{List, Reader, Writer};
#[test]
fn all_eight_account_changes_and_their_outcomes_round_trip() {
    let mut uses = List::with_capacity(16);
    for n in 0..16 {
        uses.push(if n % 2 == 0 {
            OwnerAccountUse::Party(
                OwnerAccountUseParty::new(&CEILINGS, OwnerAccountUsePartyParts { party: u64::from(n) }).unwrap(),
            )
        } else {
            OwnerAccountUse::Project(
                OwnerAccountUseProject::new(&CEILINGS, OwnerAccountUseProjectParts { project: n }).unwrap(),
            )
        })
        .unwrap();
    }
    for change in [
        AccountChange::AddKey(
            AccountChangeAddKey::new(
                &CEILINGS,
                AccountChangeAddKeyParts { provider: 3, join: Some(7), digest: [19; 32] },
            )
            .unwrap(),
        ),
        AccountChange::ReplaceKey(
            AccountChangeReplaceKey::new(&CEILINGS, AccountChangeReplaceKeyParts { source: 9, digest: [23; 32] })
                .unwrap(),
        ),
        AccountChange::RemoveKey(
            AccountChangeRemoveKey::new(&CEILINGS, AccountChangeRemoveKeyParts { source: 9 }).unwrap(),
        ),
        AccountChange::Resume(AccountChangeResume::new(&CEILINGS, AccountChangeResumeParts { source: 9 }).unwrap()),
        AccountChange::Remove(AccountChangeRemove::new(&CEILINGS, AccountChangeRemoveParts { account: 7 }).unwrap()),
        AccountChange::MoveAccount(
            AccountChangeMoveAccount::new(&CEILINGS, AccountChangeMoveAccountParts { account: 7, place: 2 }).unwrap(),
        ),
        AccountChange::MoveSource(
            AccountChangeMoveSource::new(&CEILINGS, AccountChangeMoveSourceParts { account: 7, source: 9, place: 2 })
                .unwrap(),
        ),
        AccountChange::SetUses(
            AccountChangeSetUses::new(&CEILINGS, AccountChangeSetUsesParts { account: 7, uses: Some(uses) }).unwrap(),
        ),
    ] {
        for holder in [AccountHolder::Own, AccountHolder::Deployment] {
            let ask = Ask::AccountChange(
                AskAccountChange::new(&CEILINGS, AskAccountChangeParts { holder, change: change.clone() }).unwrap(),
            );
            let mut out = Writer::new(ask.measure() as usize);
            ask.encode(&mut out).unwrap();
            assert_eq!(Ask::decode(&CEILINGS, &mut Reader::new(&out.finish())), Ok(ask));
        }
    }
    for outcome in [
        OwnerOutcome::AccountKeyKept(
            OwnerOutcomeAccountKeyKept::new(&CEILINGS, OwnerOutcomeAccountKeyKeptParts { source: u32::MAX }).unwrap(),
        ),
        OwnerOutcome::AccountChanged,
        OwnerOutcome::AccountRemoved(
            OwnerOutcomeAccountRemoved::new(&CEILINGS, OwnerOutcomeAccountRemovedParts { confirmed: true }).unwrap(),
        ),
    ] {
        let mut out = Writer::new(outcome.measure() as usize);
        outcome.encode(&mut out).unwrap();
        assert_eq!(OwnerOutcome::decode(&CEILINGS, &mut Reader::new(&out.finish())), Ok(outcome));
    }
    let mut uses = List::with_capacity(17);
    for _ in 0..17 {
        uses.push(OwnerAccountUse::Party(
            OwnerAccountUseParty::new(&CEILINGS, OwnerAccountUsePartyParts { party: 1 }).unwrap(),
        ))
        .unwrap();
    }
    assert!(AccountChangeSetUses::new(&CEILINGS, AccountChangeSetUsesParts { account: 1, uses: Some(uses) }).is_err());
    let mut out = Writer::new(Refusal::Provider.measure() as usize);
    Refusal::Provider.encode(&mut out).unwrap();
    assert_eq!(Refusal::decode(&CEILINGS, &mut Reader::new(&out.finish())), Ok(Refusal::Provider));
}
