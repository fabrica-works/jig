//! Shared card facts, acts and part slots (client/domain/objects.md, sections 2–9).
use crate::phrases::jig::JigPhrase;
use crate::words::{Words, name};
use crate::{JigBindings, JigMeaning, Owner};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run, SlotRef};

use crate::{
    Borrowed,
    action::Act,
    fields::Purpose,
    objects::{Card, Object},
};
use jig_client::contract::{ActKind, Admission, Authority, Budget, Offered, Part, Passed, Recipient, TakenBy};
use skein_lib::{Id, List};
/// The phrase for every offered act, matched whole.
#[must_use]
pub const fn act_phrase(act: ActKind) -> JigPhrase {
    match act {
        ActKind::Write => JigPhrase::ActWrite,
        ActKind::Answer => JigPhrase::ActAnswer,
        ActKind::Take => JigPhrase::ActTake,
        ActKind::HandBack => JigPhrase::ActHandBack,
        ActKind::Accept => JigPhrase::ActAccept,
        ActKind::Reject => JigPhrase::ActReject,
        ActKind::PassUp => JigPhrase::ActPassUp,
        ActKind::Release => JigPhrase::ActRelease,
        ActKind::LeaveHeld => JigPhrase::ActLeaveHeld,
        ActKind::Amend => JigPhrase::ActAmend,
        ActKind::AmendRelease => JigPhrase::ActAmendRelease,
        ActKind::Stop => JigPhrase::ActStop,
        ActKind::Cancel => JigPhrase::ActCancel,
        ActKind::Close => JigPhrase::ActClose,
        ActKind::TakeOver => JigPhrase::ActTakeOver,
        ActKind::SetGoal => JigPhrase::ActSetGoal,
        ActKind::Prioritise => JigPhrase::ActPrioritise,
        ActKind::MarkSeen => JigPhrase::ActMarkSeen,
        ActKind::EditNote => JigPhrase::ActEditNote,
        ActKind::DeleteNote => JigPhrase::ActDeleteNote,
        ActKind::Adopt => JigPhrase::ActAdopt,
        ActKind::ChangePolicy => JigPhrase::ActChangePolicy,
        ActKind::SetRoles => JigPhrase::ActSetRoles,
        ActKind::MakeService => JigPhrase::ActMakeService,
        ActKind::SetPool => JigPhrase::ActSetPool,
    }
}
/// Render only document-admitted acts on cards able to open a confirmation.
pub fn acts(
    b: &mut Builder,
    card: &Card,
    rows: &List<Offered>,
    object: Id<Object>,
    words: &Words,
    bindings: &mut JigBindings,
) {
    let text_limit = bindings.text_limit();
    match card {
        Card::Open | Card::NotSent { .. } | Card::Refused { .. } => {}
        Card::Deciding { .. } | Card::Unknown { .. } | Card::Leaving { .. } => return,
    }
    for offered in rows {
        match &offered.admission {
            Admission::Allowed | Admission::Proposed { .. } | Admission::Bounded { .. } => crate::frame::control(
                b,
                words,
                bindings,
                Role::Button,
                act_phrase(offered.act),
                &[],
                Act::Intend { object: object.token(), act: offered.act },
            ),
            Admission::Refused(why) => {
                words.text(b, act_phrase(offered.act), &[]);
                words.refusal(b, why, text_limit);
            }
        }
    }
}
/// Leave a slot and retain the typed part that its owning view root will resolve.
pub fn part(b: &mut Builder, part: &Part, words: &Words, bindings: &mut JigBindings) {
    let connector = match part {
        Part::Connector { connector }
        | Part::Resource { connector, .. }
        | Part::Kind { connector, .. }
        | Part::Panel { connector, .. }
        | Part::Result { connector, .. }
        | Part::Effect { connector, .. }
        | Part::OwnerEffect { connector, .. }
        | Part::Outbox { connector, .. }
        | Part::Hold { connector, .. }
        | Part::Project { connector, .. }
        | Part::Deployment { connector } => *connector,
    };
    let token = bindings.insert(JigMeaning::Part(part.clone())).token;
    b.slot(
        SlotRef { owner: (Owner::Application { number: connector }).opaque(), token },
        &words.say(JigPhrase::LoadingPart, &[]),
    );
}
/// Closing keeps only engine-owned delegate counts and the connectors it waits on.
/// The connector's words fill a name-only part; effect details remain in its task panel.
pub fn closing(b: &mut Builder, closing: &jig_client::contract::Closing, words: &Words, bindings: &mut JigBindings) {
    words.text(b, JigPhrase::Closing, &[Hole::Number(u64::from(closing.delegates))]);
    match closing.stage {
        Some(jig_client::contract::ClosingStage::Run { attempt }) => {
            words.text(b, JigPhrase::ClosingRun, &[Hole::Number(attempt)]);
        }
        Some(jig_client::contract::ClosingStage::Delegates) => words.text(b, JigPhrase::ClosingDelegates, &[]),
        Some(jig_client::contract::ClosingStage::Effects) => words.text(b, JigPhrase::ClosingStageEffects, &[]),
        Some(jig_client::contract::ClosingStage::Releases) => words.text(b, JigPhrase::ClosingReleases, &[]),
        Some(jig_client::contract::ClosingStage::Settled) => words.text(b, JigPhrase::ClosingSettled, &[]),
        None => words.text(b, JigPhrase::ClosingStageUnknown, &[]),
    }
    if let Some(ending) = &closing.ending {
        words.text(b, JigPhrase::ClosingPendingEnding, &[]);
        words.ending(b, ending);
    }
    match &closing.connectors {
        Some(connectors) => {
            if connectors.is_empty() {
                words.text(b, JigPhrase::ClosingConnectorsEmpty, &[]);
            }
            for connector in connectors {
                words.text(b, JigPhrase::ClosingConnector, &[]);
                part(b, &Part::Connector { connector: *connector }, words, bindings);
                if closing.stage == Some(jig_client::contract::ClosingStage::Effects) {
                    words.text(b, JigPhrase::ClosingEffects, &[]);
                }
            }
        }
        None => words.text(b, JigPhrase::ClosingConnectorsUnknown, &[]),
    }
}
/// Say where an object is waiting, without deriving its policy recipient.
pub fn recipient(b: &mut Builder, to: &Recipient, words: &Words) {
    match to {
        Recipient::Task(task) => words.text(b, JigPhrase::WaitingTask, &[Hole::Number(*task)]),
        Recipient::Party(party) => words.text(b, JigPhrase::WaitingParty, &[Hole::Number(*party)]),
        Recipient::Role { project, role } => {
            words.text(b, JigPhrase::WaitingRole, &[Hole::Number(u64::from(*project)), Hole::Number(u64::from(*role))]);
        }
        Recipient::Roles { project, roles } => {
            words.text(b, JigPhrase::WaitingRoles, &[Hole::Number(u64::from(*project))]);
            for role in roles {
                words.text(b, JigPhrase::RoleNumber, &[Hole::Number(u64::from(*role))]);
            }
        }
    }
}

pub fn taken_by(b: &mut Builder, by: &TakenBy, words: &Words) {
    match &by.name {
        Some(value) => words.text(b, JigPhrase::TakenBy, &[name(value.bytes())]),
        None => words.text(b, JigPhrase::TakenByNumber, &[Hole::Number(by.number)]),
    }
}
/// Show the whole authority facts, with grants' connector descriptions in slots.
pub fn authority(b: &mut Builder, facts: &Authority, words: &Words, bindings: &mut JigBindings) {
    let text_limit = bindings.text_limit();
    words.text(b, JigPhrase::Budget, &[Hole::Amount(facts.budget)]);
    words.text(b, JigPhrase::Depth, &[Hole::Number(u64::from(facts.depth))]);
    words.text(b, JigPhrase::TasksAllowed, &[Hole::Number(u64::from(facts.tasks))]);
    if let Some(at) = facts.deadline {
        words.deadline(b, at);
    }
    for executor in &facts.task_kinds {
        words.executor(b, *executor);
    }
    for grant in &facts.grants {
        words.text(
            b,
            JigPhrase::LiteralGrant,
            &[Hole::Number(u64::from(grant.kind)), Hole::Number(u64::from(grant.connector))],
        );
        crate::owner_authority::pattern(b, words, &grant.pattern, text_limit);
        part(b, &Part::Kind { connector: grant.connector, kind: grant.kind }, words, bindings);
    }
    for tool in &facts.tools {
        words.text(b, JigPhrase::ToolAllowed, &[Hole::Number(u64::from(*tool))]);
    }
    for scope in &facts.note_scopes {
        match scope {
            jig_client::contract::NoteScope::Deployment => words.text(b, JigPhrase::Deployment, &[]),
            jig_client::contract::NoteScope::Project(project) => {
                words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(*project))]);
            }
            jig_client::contract::NoteScope::Resource { project, connector, pattern } => {
                words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(*project))]);
                words.text(b, JigPhrase::PartConnector, &[Hole::Number(u64::from(*connector))]);
                crate::owner_authority::pattern(b, words, pattern, text_limit);
            }
            jig_client::contract::NoteScope::Goal { project, goal } => {
                words.text(b, JigPhrase::ContextProject, &[Hole::Number(u64::from(*project))]);
                words.text(b, JigPhrase::TaskPage, &[Hole::Number(*goal)]);
            }
        }
    }
}
/// Funding facts as given; checked subtraction detects an inconsistent document.
pub fn funding(b: &mut Builder, facts: &Budget, amount: u64, words: &Words) {
    match facts.left.checked_sub(amount) {
        Some(after) => {
            words.text(b, JigPhrase::Funds, &[Hole::Amount(amount), Hole::Amount(facts.left), Hole::Amount(after)]);
        }
        None => words.text(b, JigPhrase::BudgetMissing, &[Hole::Amount(facts.left), Hole::Amount(amount)]),
    }
    words.actor(b, facts.funder);
}
/// Each prior holder and its committed reason and missing authority.
pub fn trail(b: &mut Builder, rows: &List<Passed>, words: &Words, text_limit: u32) {
    for passed in rows {
        words.text(b, JigPhrase::PassedBy, &[]);
        words.actor(b, passed.by);
        words.wall(b, passed.at);
        b.text(Run::Plain, passed.reason.bytes());
        if let Some(lacking) = &passed.lacking {
            words.lacking(b, lacking, text_limit);
        }
    }
}
/// Render a form field with its domain write count; the shell preserves typing.
pub fn field(
    b: &mut Builder,
    client: &Borrowed<'_>,
    purpose: Purpose,
    phrase: JigPhrase,
    words: &Words,
    bindings: &mut JigBindings,
) {
    b.open(Role::TextArea);
    b.name(&words.say(phrase, &[]));
    if let Some(field) = client.fields().get(purpose) {
        b.value(field.text.bytes(), field.writes);
    } else {
        b.value(&[], 0);
    }
    b.bind(bindings.insert(JigMeaning::Field(purpose)));
    b.close();
}
/// Required fields and choices are visible once this object's confirmation is open.
#[must_use]
pub fn answering(client: &Borrowed<'_>, object: Id<Object>) -> bool {
    match client.confirmation() {
        Some(open) => open.object == object && open.act == ActKind::Answer,
        None => false,
    }
}
/// Display any kept durable answer even on a card whose object remains open.
pub fn answer(b: &mut Builder, object: &Object, words: &Words, text_limit: u32) {
    if let Some(answer) = &object.answer {
        match answer {
            jig_client::contract::Answer::Done(outcome) => words.outcome(b, outcome),
            jig_client::contract::Answer::Refused(why) => words.refusal(b, why, text_limit),
        }
    }
}
