//! Standalone owner authority fragments (client/domain/view.md, 7 and 10).
//! Retains literal boundaries; words are selected checked language data.
use crate::{
    Builder, Role, Run,
    owner_authority_words::{AuthorityPhrase, OwnerAuthorityWords},
    phrases::Hole,
};
use jig_client::contract::{
    OwnerAuthority, OwnerAuthorityDocument, OwnerAuthorityExecutor, OwnerPattern, OwnerPatternEnd,
};

/// Storage needed for one standalone authority region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthorityFragment {
    pub nodes: u32,
    pub text_bytes: u32,
    pub text_per_node: u32,
    pub depth: u32,
}
impl AuthorityFragment {
    fn phrase(&mut self, words: &OwnerAuthorityWords, phrase: AuthorityPhrase) -> Option<()> {
        self.leaf(words.capacity(phrase))
    }
    fn leaf(&mut self, bytes: u32) -> Option<()> {
        self.nodes = self.nodes.checked_add(1)?;
        self.text_bytes = self.text_bytes.checked_add(bytes)?;
        self.text_per_node = self.text_per_node.max(bytes);
        Some(())
    }
    fn pattern(&mut self, pattern: &OwnerPattern, words: &OwnerAuthorityWords) -> Option<()> {
        self.phrase(words, AuthorityPhrase::Segments)?;
        for segment in &pattern.segments {
            self.leaf(u32::try_from(segment.len()).ok()?)?;
        }
        match &pattern.last {
            OwnerPatternEnd::Exact(bytes) => {
                self.phrase(words, AuthorityPhrase::Exact)?;
                self.leaf(u32::try_from(bytes.len()).ok()?)?;
            }
            OwnerPatternEnd::Open(bytes) => {
                self.phrase(words, AuthorityPhrase::Open)?;
                self.leaf(u32::try_from(bytes.len()).ok()?)?;
            }
        }
        Some(())
    }
}
/// Checked fragment capacities; phrase capacities cover every loaded language.
#[must_use]
pub fn required(value: &OwnerAuthority, words: &OwnerAuthorityWords) -> Option<AuthorityFragment> {
    let mut result = AuthorityFragment { nodes: 1, text_bytes: 0, text_per_node: 0, depth: 2 };
    for phrase in [
        AuthorityPhrase::Tools,
        AuthorityPhrase::Tasks,
        AuthorityPhrase::Depth,
        AuthorityPhrase::Spend,
        AuthorityPhrase::Notes,
        AuthorityPhrase::Grants,
        AuthorityPhrase::Delegation,
        AuthorityPhrase::Resources,
    ] {
        result.phrase(words, phrase)?;
    }
    if value.deadline.is_some() {
        result.phrase(words, AuthorityPhrase::Deadline)?;
    }
    for grant in &value.grants {
        result.phrase(words, AuthorityPhrase::Connector)?;
        result.phrase(words, AuthorityPhrase::Kind)?;
        result.pattern(&grant.pattern, words)?;
    }
    for kind in &value.delegation.kinds {
        let phrase = match kind {
            OwnerAuthorityExecutor::Charter(_) => AuthorityPhrase::Charter,
            OwnerAuthorityExecutor::Procedure(_) => AuthorityPhrase::Procedure,
            OwnerAuthorityExecutor::Role(_) => AuthorityPhrase::Role,
        };
        result.phrase(words, phrase)?;
    }
    for resource in &value.note_resources {
        result.phrase(words, AuthorityPhrase::Connector)?;
        result.pattern(&resource.pattern, words)?;
    }
    Some(result)
}
/// Include the owner's cut fact without inferring completeness from other facts.
#[must_use]
pub fn document_required(value: &OwnerAuthorityDocument, words: &OwnerAuthorityWords) -> Option<AuthorityFragment> {
    let mut result = required(&value.authority, words)?;
    if value.cut {
        result.phrase(words, AuthorityPhrase::Cut)?;
    }
    Some(result)
}
/// Configured envelope for full rows of grants, resource scopes and pattern segments.
#[must_use]
pub fn worst_case(rows: u32, text: u32, words: &OwnerAuthorityWords) -> Option<AuthorityFragment> {
    let mut phrase = 0;
    for id in crate::owner_authority_words::ALL {
        phrase = phrase.max(words.capacity(id));
    }
    let patterns = rows.checked_mul(2)?;
    let raw = rows.checked_add(1)?.checked_mul(patterns)?;
    let sayings = rows.checked_mul(8)?.checked_add(10)?;
    let nodes = raw.checked_add(sayings)?.checked_add(1)?;
    let text_bytes = raw.checked_mul(text)?.checked_add(sayings.checked_mul(phrase)?)?;
    Some(AuthorityFragment { nodes, text_bytes, text_per_node: text.max(phrase), depth: 2 })
}
fn text(b: &mut Builder, words: &OwnerAuthorityWords, phrase: AuthorityPhrase, holes: &[Hole]) {
    b.text(Run::Plain, &words.say(phrase, holes));
}
fn number(b: &mut Builder, words: &OwnerAuthorityWords, phrase: AuthorityPhrase, value: u64) {
    text(b, words, phrase, &[Hole::Number(value)]);
}
fn pattern(b: &mut Builder, words: &OwnerAuthorityWords, value: &OwnerPattern) {
    text(b, words, AuthorityPhrase::Segments, &[]);
    for segment in &value.segments {
        b.text(Run::Code, segment);
    }
    match &value.last {
        OwnerPatternEnd::Exact(bytes) => {
            text(b, words, AuthorityPhrase::Exact, &[]);
            b.text(Run::Code, bytes);
        }
        OwnerPatternEnd::Open(bytes) => {
            text(b, words, AuthorityPhrase::Open, &[]);
            b.text(Run::Code, bytes);
        }
    }
}
fn body(b: &mut Builder, value: &OwnerAuthority, words: &OwnerAuthorityWords) {
    number(b, words, AuthorityPhrase::Tools, value.tools);
    number(b, words, AuthorityPhrase::Tasks, u64::from(value.delegation.tasks));
    number(b, words, AuthorityPhrase::Depth, u64::from(value.delegation.depth));
    text(b, words, AuthorityPhrase::Spend, &[Hole::Amount(value.spend)]);
    number(b, words, AuthorityPhrase::Notes, u64::from(value.notes));
    if let Some(deadline) = value.deadline {
        text(b, words, AuthorityPhrase::Deadline, &[Hole::Wall(words.adjusted_wall(deadline))]);
    }
    text(b, words, AuthorityPhrase::Grants, &[]);
    for grant in &value.grants {
        number(b, words, AuthorityPhrase::Connector, u64::from(grant.connector));
        number(b, words, AuthorityPhrase::Kind, u64::from(grant.kind));
        pattern(b, words, &grant.pattern);
    }
    text(b, words, AuthorityPhrase::Delegation, &[]);
    for kind in &value.delegation.kinds {
        match kind {
            OwnerAuthorityExecutor::Charter(code) => number(b, words, AuthorityPhrase::Charter, u64::from(*code)),
            OwnerAuthorityExecutor::Procedure(code) => number(b, words, AuthorityPhrase::Procedure, u64::from(*code)),
            OwnerAuthorityExecutor::Role(code) => number(b, words, AuthorityPhrase::Role, u64::from(*code)),
        }
    }
    text(b, words, AuthorityPhrase::Resources, &[]);
    for resource in &value.note_resources {
        number(b, words, AuthorityPhrase::Connector, u64::from(resource.connector));
        pattern(b, words, &resource.pattern);
    }
}
/// Render a whole admitted payload using a separately reserved fragment capacity.
pub fn render(b: &mut Builder, value: &OwnerAuthority, words: &OwnerAuthorityWords) {
    b.open(Role::Region);
    body(b, value, words);
    b.close();
}
/// Render a document projection and its explicit cut metadata.
pub fn document(b: &mut Builder, value: &OwnerAuthorityDocument, words: &OwnerAuthorityWords) {
    b.open(Role::Region);
    body(b, &value.authority, words);
    if value.cut {
        text(b, words, AuthorityPhrase::Cut, &[]);
    }
    b.close();
}
