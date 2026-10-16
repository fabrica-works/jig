//! Explicitly scoped owner fixtures, with checked whole-document admission.
use jig_client_domain::contract::Revision;
use jig_client_scripted_engine::{Application, ApplicationValue, Bounds, Change, Decision, Truth};
use jig_ops_client_domain::{infrastructure as infra, observability as obs};
use skein_lib::Wall;
use std::collections::BTreeMap;

/// The two connector subjects retain their original scope and numeric identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Subject {
    Observability(obs::Subject),
    Infrastructure(infra::Subject),
}
/// Owner documents; no link or price is inferred by this adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Document {
    Observability(obs::Document),
    Infrastructure(infra::Document),
}
/// Finite Ops fixtures beside jig's supplied truth.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpsScript {
    pub documents: BTreeMap<Subject, Document>,
}
impl Application for OpsScript {
    type Ask = infra::Ask;
    type Answer = infra::Answer;
    type Subject = Subject;
    type Document = Document;
    fn decide(&mut self, _: &Truth, _: u64, ask: &infra::Ask, _: Wall) -> Decision<infra::Answer> {
        let infra::Ask::MoveEnd { project, environment, revision, until } = ask else {
            return Decision {
                answer: infra::Answer::Refused { why: infra::documents::Refusal::Invalid },
                changes: vec![],
            };
        };
        let subject = Subject::Infrastructure(infra::Subject {
            scope: infra::Scope::Project(*project),
            kind: infra::Kind::Environment(*environment),
        });
        let answer = match self.documents.get_mut(&subject) {
            Some(Document::Infrastructure(infra::Document::Environment(env))) => match env.end {
                Some(end) if end.revision == *revision => {
                    let revision =
                        Revision { value: end.revision.value.checked_add(1).expect("finite fixture revision") };
                    env.revision.value = env.revision.value.checked_add(1).expect("finite document revision");
                    env.end = Some(infra::documents::End { revision, until: *until });
                    // The actual backend lease stays supplied, even while desired end advances.
                    infra::Answer::MovedEnd { environment: *environment, revision, until: *until }
                }
                Some(end) => {
                    infra::Answer::Moved { environment: *environment, revision: end.revision, until: Some(end.until) }
                }
                None => infra::Answer::Gone,
            },
            Some(Document::Observability(_) | Document::Infrastructure(_)) | None => infra::Answer::Gone,
        };
        Decision { answer, changes: vec![] }
    }
    fn document(&self, truth: &Truth, party: u64, subject: &Subject) -> Option<Document> {
        let project = match subject {
            Subject::Observability(obs::Subject { scope: obs::Scope::Project(project), .. })
            | Subject::Infrastructure(infra::Subject { scope: infra::Scope::Project(project), .. }) => Some(*project),
            Subject::Observability(obs::Subject { scope: obs::Scope::Deployment, .. })
            | Subject::Infrastructure(infra::Subject { scope: infra::Scope::Deployment, .. }) => None,
        };
        let frame = truth.frames.get(&party)?;
        let visible = project.map_or_else(
            || frame.deployment_role.is_some(),
            |project| frame.projects.iter().any(|row| row.project == project),
        );
        visible.then(|| self.documents.get(subject).cloned()).flatten()
    }
    fn marks(&self, _: &Change) -> Vec<Subject> {
        self.documents.keys().cloned().collect()
    }
    fn fits(&self, value: ApplicationValue<'_, Self>, bounds: &Bounds) -> bool {
        let mut limits = super::limits();
        limits.observability.rows = bounds.rows;
        limits.observability.text = bounds.text;
        limits.observability.document_bytes = bounds.document_bytes;
        limits.infrastructure.rows = bounds.rows;
        limits.infrastructure.text = bounds.text;
        limits.infrastructure.document_bytes = bounds.document_bytes;
        let fits = |doc: &Document| match doc {
            Document::Observability(doc) => doc.fits(&limits.observability),
            Document::Infrastructure(doc) => doc.fits(&limits.infrastructure),
        };
        match value {
            ApplicationValue::Document(doc) => fits(doc),
            ApplicationValue::State(state) => state.documents.iter().all(|(subject, doc)| {
                let same = match (subject, doc) {
                    (Subject::Observability(subject), Document::Observability(doc)) => *subject == doc.subject(),
                    (Subject::Infrastructure(subject), Document::Infrastructure(doc)) => *subject == doc.subject(),
                    (Subject::Observability(_), Document::Infrastructure(_))
                    | (Subject::Infrastructure(_), Document::Observability(_)) => false,
                };
                same && fits(doc)
            }),
            ApplicationValue::Ask(_) | ApplicationValue::Answer(_) | ApplicationValue::Subject(_) => true,
        }
    }
}
