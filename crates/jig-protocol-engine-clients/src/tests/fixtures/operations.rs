use super::*;

pub fn answer(on: w::AnswerOn) -> w::Ask {
    w::Ask::Answer(w::Answer::new(&wire(), w::AnswerParts { project: 3, on }).expect("answer envelope"))
}
pub fn decide(on: w::DecideOn) -> w::Ask {
    w::Ask::Decide(w::Decide::new(&wire(), w::DecideParts { project: 3, on }).expect("decision envelope"))
}
pub fn empty_amendment(tracked: Option<w::Tracking>) -> w::Amendment {
    w::Amendment::new(
        &wire(),
        w::AmendmentParts {
            addressed: None,
            spec: None,
            wake: None,
            dependencies: None,
            authority: None,
            instructions: None,
            tracked,
        },
    )
    .expect("empty amendment")
}
pub fn priority_amend() -> w::Ask {
    let tracked = w::Tracking::Tracked(
        w::TrackingTracked::new(&wire(), w::TrackingTrackedParts { priority: u32::MAX }).expect("priority"),
    );
    w::Ask::Amend(Box::new(
        w::Amend::new(
            &wire(),
            w::AmendParts {
                project: 3,
                task: 8,
                revision: None,
                change: empty_amendment(Some(tracked)),
                reason: Box::new([]),
            },
        )
        .expect("priority amendment"),
    ))
}
pub fn template_change() -> w::Ask {
    let value = w::TemplateOverride::new(
        &wire(),
        w::TemplateOverrideParts {
            template: 2,
            executor: Some(w::TemplateExecutor::Chosen),
            budget: Some(w::TemplateBudget::Given),
            deadline: Some(w::TemplateDeadline::None),
            priority: Some(u32::MAX),
        },
    )
    .expect("override");
    let mut overrides = List::with_capacity(1);
    overrides.push(value).expect("one override");
    let edit = w::PolicyEdit::Templates(
        w::PolicyEditTemplates::new(&wire(), w::PolicyEditTemplatesParts { overrides }).expect("templates"),
    );
    w::Ask::ChangePolicy(
        w::ChangePolicy::new(&wire(), w::ChangePolicyParts { project: 3, edit }).expect("change policy"),
    )
}

/// An amendment retaining every field, with the supplied explanation.
#[must_use]
pub fn person_unchanged(reason: &[u8]) -> jig_domain_people::Amendment {
    jig_domain_people::Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        reason: reason.into(),
    }
}

pub fn compound(project: u32, values: &[w::Operation]) -> w::Ask {
    let mut size = 0_usize;
    for value in values {
        size = size.checked_add(usize::try_from(value.measure()).unwrap()).unwrap().checked_add(4).unwrap();
    }
    let mut bytes = Writer::new(size);
    for value in values {
        bytes.put(&value.measure().to_be_bytes()).unwrap();
        value.encode(&mut bytes).unwrap();
    }
    w::Ask::Compound(w::Compound::new(&wire(), w::CompoundParts { project, operations: bytes.finish() }).unwrap())
}
