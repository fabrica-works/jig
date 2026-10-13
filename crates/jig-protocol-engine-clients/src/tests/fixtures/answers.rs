use super::*;
use jig_domain_people as p;
#[path = "answers/outcomes.rs"]
mod outcomes;
#[path = "answers/refusals.rs"]
mod refusals;
pub use outcomes::*;
pub use refusals::*;
fn done(outcome: w::Outcome) -> w::Final {
    w::Final::Done(w::Done::new(&wire(), w::DoneParts { outcome }).expect("done"))
}
fn owner(value: w::OwnerOutcome) -> w::Final {
    done(w::Outcome::Owner(w::OutcomeOwner::new(&wire(), w::OutcomeOwnerParts { value }).expect("owner outcome")))
}
pub fn started() -> w::Final {
    owner(w::OwnerOutcome::Started(
        w::OwnerOutcomeStarted::new(&wire(), w::OwnerOutcomeStartedParts { task: 8 }).expect("started"),
    ))
}
pub fn moved_hold() -> w::Final {
    let reason = w::HoldCategory::Dependency(
        w::HoldCategoryDependency::new(&wire(), w::HoldCategoryDependencyParts { task: 9 }).expect("dependency"),
    );
    w::Final::Refused(w::Refused::new(&wire(), w::RefusedParts { refusal: hold_refusal(reason) }).expect("refused"))
}
pub const SERVICE_ANSWER: [u8; 25] = [0, 0, 0, 0, 0, 6, 0, 0, 0, 15, 0, 1, 4, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0];
pub fn application_answer() -> w::Answer {
    let owner = w::Owner::Family(w::Family::new(&wire(), w::FamilyParts { number: 9 }).expect("family"));
    w::Answer::FinalAnswer(
        w::Owned::new(&wire(), w::OwnedParts { owner, version: 2, record: Box::from(b"outcome".as_slice()) })
            .expect("application answer"),
    )
}

#[path = "answers/admission.rs"]
mod admission;
#[path = "answers/lacks.rs"]
mod lacks;
pub use admission::*;
pub use lacks::*;

pub fn response_bounds() -> [(u32, Option<w::Answer>); 2] {
    let limits = w::Limits { owned_record: 512, ..wire() };
    let owner = w::Owner::Family(w::Family::new(&limits, w::FamilyParts { number: 9 }).expect("family"));
    let answer = w::Answer::FinalAnswer(
        w::Owned::new(&limits, w::OwnedParts { owner, version: 2, record: Box::new([3; 257]) })
            .expect("body fits its record bound"),
    );
    [(512, Some(answer)), (1, None)]
}
