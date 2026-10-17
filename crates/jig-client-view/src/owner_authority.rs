//! Standalone owner authority fragments (client/domain/view.md, 7 and 10).
//! Retains literal boundaries; words are selected checked language data.
use crate::owner_authority_words::{AuthorityPhrase, OwnerAuthorityWords};
use crate::words::Words;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

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
    fn phrase(&mut self, words: &Words, phrase: AuthorityPhrase) -> Option<()> {
        self.leaf(words.book.capacity(phrase.id())?)
    }
    fn leaf(&mut self, bytes: u32) -> Option<()> {
        self.nodes = self.nodes.checked_add(1)?;
        self.text_bytes = self.text_bytes.checked_add(bytes)?;
        self.text_per_node = self.text_per_node.max(bytes);
        Some(())
    }
    fn pattern(&mut self, pattern: &OwnerPattern, words: &Words) -> Option<()> {
        self.phrase(words, AuthorityPhrase::Segments)?;
        for segment in &pattern.segments {
            self.leaf(crate::literal_segment::display_bytes(segment)?)?;
        }
        match &pattern.last {
            OwnerPatternEnd::Exact(bytes) => {
                self.phrase(words, AuthorityPhrase::Exact)?;
                self.leaf(crate::literal_segment::display_bytes(bytes)?)?;
            }
            OwnerPatternEnd::Open(bytes) => {
                self.phrase(words, AuthorityPhrase::Open)?;
                self.leaf(crate::literal_segment::display_bytes(bytes)?)?;
            }
        }
        Some(())
    }
}
/// Checked fragment capacities; phrase capacities cover every loaded language.
#[must_use]
pub fn required_words(value: &OwnerAuthority, words: &Words) -> Option<AuthorityFragment> {
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
pub fn document_required_words(value: &OwnerAuthorityDocument, words: &Words) -> Option<AuthorityFragment> {
    let mut result = required_words(&value.authority, words)?;
    if value.cut {
        result.phrase(words, AuthorityPhrase::Cut)?;
    }
    Some(result)
}
/// Configured envelope for full rows of grants, resource scopes and pattern segments.
#[must_use]
pub fn worst_case_words(rows: u32, text: u32, words: &Words) -> Option<AuthorityFragment> {
    let mut phrase = 0;
    for id in crate::owner_authority_words::ALL {
        phrase = phrase.max(words.book.capacity(id.id())?);
    }
    let patterns = rows.checked_mul(2)?;
    let raw = rows.checked_add(1)?.checked_mul(patterns)?;
    let sayings = rows.checked_mul(8)?.checked_add(10)?;
    let nodes = document_nodes(rows)?;
    let text = crate::literal_segment::max_bytes(text)?;
    let text_bytes = raw.checked_mul(text)?.checked_add(sayings.checked_mul(phrase)?)?;
    Some(AuthorityFragment { nodes, text_bytes, text_per_node: text.max(phrase), depth: 2 })
}
fn text(b: &mut Builder, words: &Words, phrase: AuthorityPhrase, holes: &[Hole]) {
    let saying = words.book.say(words.language, phrase.id(), holes).expect("checked authority words and facts");
    b.text(Run::Plain, &saying.text);
}
fn number(b: &mut Builder, words: &Words, phrase: AuthorityPhrase, value: u64) {
    text(b, words, phrase, &[Hole::Number(value)]);
}
pub(crate) fn pattern(b: &mut Builder, words: &Words, value: &OwnerPattern) {
    text(b, words, AuthorityPhrase::Segments, &[]);
    for segment in &value.segments {
        crate::literal_segment::render(b, segment);
    }
    match &value.last {
        OwnerPatternEnd::Exact(bytes) => {
            text(b, words, AuthorityPhrase::Exact, &[]);
            crate::literal_segment::render(b, bytes);
        }
        OwnerPatternEnd::Open(bytes) => {
            text(b, words, AuthorityPhrase::Open, &[]);
            crate::literal_segment::render(b, bytes);
        }
    }
}
fn body(b: &mut Builder, value: &OwnerAuthority, words: &Words) {
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
pub fn render_words(b: &mut Builder, value: &OwnerAuthority, words: &Words) {
    b.open(Role::Description);
    body(b, value, words);
    b.close();
}
/// Render a document projection and its explicit cut metadata.
pub fn document_words(b: &mut Builder, value: &OwnerAuthorityDocument, words: &Words) {
    b.open(Role::Description);
    body(b, &value.authority, words);
    if value.cut {
        text(b, words, AuthorityPhrase::Cut, &[]);
    }
    b.close();
}

/// Maximum nodes for a supplied authority document, including its cut marker.
#[must_use]
pub fn document_nodes(rows: u32) -> Option<u32> {
    let raw = rows.checked_add(1)?.checked_mul(rows.checked_mul(2)?)?;
    raw.checked_add(rows.checked_mul(8)?.checked_add(10)?)?.checked_add(1)
}
/// Standalone capacity API shares the original checked book.
#[must_use]
pub fn required(value: &OwnerAuthority, words: &OwnerAuthorityWords) -> Option<AuthorityFragment> {
    required_words(value, words.words())
}
/// Standalone document capacity API shares the original checked book.
#[must_use]
pub fn document_required(value: &OwnerAuthorityDocument, words: &OwnerAuthorityWords) -> Option<AuthorityFragment> {
    document_required_words(value, words.words())
}
/// Standalone configured envelope includes escaped literal presentation.
#[must_use]
pub fn worst_case(rows: u32, text: u32, words: &OwnerAuthorityWords) -> Option<AuthorityFragment> {
    worst_case_words(rows, text, words.words())
}
/// Render a standalone authority with its selected checked language.
pub fn render(b: &mut Builder, value: &OwnerAuthority, words: &OwnerAuthorityWords) {
    render_words(b, value, words.words());
}
/// Render a standalone authority document and its supplied cut marker.
pub fn document(b: &mut Builder, value: &OwnerAuthorityDocument, words: &OwnerAuthorityWords) {
    document_words(b, value, words.words());
}
