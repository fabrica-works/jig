//! Owner budget and ledger fragments (client/domain/view.md, sections 7 and 10).
//! All accounting and period facts are supplied by the owner; this view copies
//! them into the selected language, without deriving availability or a calendar.
use crate::owner_budget_words::BudgetPhrase;
use crate::proposed_task;
use crate::words::Words;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use jig_domain_client::contract::{OwnerBudget, OwnerNumbers, PoolLedger};
use skein_lib::Wall;

/// Checked tree storage for one standalone budget or ledger region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BudgetFragment {
    pub nodes: u32,
    pub text_bytes: u32,
    pub text_per_node: u32,
    pub depth: u32,
}
impl BudgetFragment {
    fn phrase(&mut self, words: &Words, phrase: BudgetPhrase) -> Option<()> {
        self.leaf(words.book.capacity(phrase.id())?)
    }
    fn name(&mut self, words: &Words, phrase: BudgetPhrase) -> Option<()> {
        let bytes = words.book.capacity(phrase.id())?;
        self.text_bytes = self.text_bytes.checked_add(bytes)?;
        self.text_per_node = self.text_per_node.max(bytes);
        Some(())
    }
    fn leaf(&mut self, bytes: u32) -> Option<()> {
        self.nodes = self.nodes.checked_add(1)?;
        self.text_bytes = self.text_bytes.checked_add(bytes)?;
        self.text_per_node = self.text_per_node.max(bytes);
        Some(())
    }
    fn funder(&mut self, words: &Words) -> Option<()> {
        let bytes = proposed_task::funder_phrase_bytes(words)?;
        for _ in 0..proposed_task::funder_nodes() {
            self.leaf(bytes)?;
        }
        Some(())
    }
    fn numbers(&mut self, words: &Words) -> Option<()> {
        for phrase in [
            BudgetPhrase::Budget,
            BudgetPhrase::Spent,
            BudgetPhrase::SpentBelow,
            BudgetPhrase::Reserved,
            BudgetPhrase::Left,
        ] {
            self.phrase(words, phrase)?;
        }
        Some(())
    }
}
fn region() -> BudgetFragment {
    BudgetFragment { nodes: 1, text_bytes: 0, text_per_node: 0, depth: 2 }
}
/// Maximum whole budget region: five numbers, the funder and an optional deadline.
#[must_use]
pub const fn budget_nodes() -> u32 {
    9
}

/// Whole supplied budget's storage, using capacities across all loaded languages.
#[must_use]
pub fn required(value: &OwnerBudget, words: &Words) -> Option<BudgetFragment> {
    let mut result = region();
    result.name(words, BudgetPhrase::Budget)?;
    result.numbers(words)?;
    result.phrase(words, BudgetPhrase::Funder)?;
    result.funder(words)?;
    if value.deadline.is_some() {
        result.phrase(words, BudgetPhrase::Deadline)?;
    }
    Some(result)
}
/// Whole supplied ledger's storage; absent optional facts add no fabricated text.
#[must_use]
pub fn ledger_required(value: &PoolLedger, words: &Words) -> Option<BudgetFragment> {
    let mut result = region();
    result.name(words, BudgetPhrase::Source)?;
    result.numbers(words)?;
    result.phrase(words, BudgetPhrase::Source)?;
    result.funder(words)?;
    if value.parent.is_some() {
        result.phrase(words, BudgetPhrase::Parent)?;
        result.funder(words)?;
    }
    result.phrase(words, if value.closed { BudgetPhrase::Closed } else { BudgetPhrase::Open })?;
    if value.began.is_some() {
        result.phrase(words, BudgetPhrase::Began)?;
    }
    if value.next.is_some() {
        result.phrase(words, BudgetPhrase::Next)?;
    }
    Some(result)
}
/// Maximum budget storage, including a supplied deadline and any exact Funder.
#[must_use]
pub fn worst_case(words: &Words) -> Option<BudgetFragment> {
    let mut result = region();
    result.name(words, BudgetPhrase::Budget)?;
    result.numbers(words)?;
    result.phrase(words, BudgetPhrase::Funder)?;
    result.funder(words)?;
    result.phrase(words, BudgetPhrase::Deadline)?;
    Some(result)
}
/// Maximum ledger storage includes both sources, both times and either state.
#[must_use]
pub fn ledger_worst_case(words: &Words) -> Option<BudgetFragment> {
    let mut result = region();
    result.name(words, BudgetPhrase::Source)?;
    result.numbers(words)?;
    result.phrase(words, BudgetPhrase::Source)?;
    result.funder(words)?;
    result.phrase(words, BudgetPhrase::Parent)?;
    result.funder(words)?;
    let open = words.book.capacity(BudgetPhrase::Open.id())?;
    let closed = words.book.capacity(BudgetPhrase::Closed.id())?;
    result.leaf(open.max(closed))?;
    result.phrase(words, BudgetPhrase::Began)?;
    result.phrase(words, BudgetPhrase::Next)?;
    Some(result)
}

fn text(b: &mut Builder, words: &Words, phrase: BudgetPhrase, holes: &[Hole]) {
    let text =
        words.book.say(words.language, phrase.id(), holes).expect("checked budget phrase and typed owner facts").text;
    b.text(Run::Plain, &text);
}
fn numbers(b: &mut Builder, value: &OwnerNumbers, left: u64, words: &Words) {
    text(b, words, BudgetPhrase::Budget, &[Hole::Amount(value.budget)]);
    text(b, words, BudgetPhrase::Spent, &[Hole::Amount(value.spent)]);
    text(b, words, BudgetPhrase::SpentBelow, &[Hole::Amount(value.spent_below)]);
    text(b, words, BudgetPhrase::Reserved, &[Hole::Amount(value.reserved)]);
    text(b, words, BudgetPhrase::Left, &[Hole::Amount(left)]);
}
fn time(b: &mut Builder, words: &Words, phrase: BudgetPhrase, at: Wall) {
    text(b, words, phrase, &[Hole::Wall(words.adjusted_wall(at))]);
}

/// Render one exact budget with the caller's normal selected Words.
pub fn render(b: &mut Builder, value: &OwnerBudget, words: &Words) {
    b.open(Role::Region);
    let label = words
        .book
        .say(words.language, BudgetPhrase::Budget.id(), &[Hole::Amount(value.numbers.budget)])
        .expect("checked budget region label")
        .text;
    b.name(&label);
    numbers(b, &value.numbers, value.left, words);
    text(b, words, BudgetPhrase::Funder, &[]);
    proposed_task::funder(b, value.funder, words);
    if let Some(at) = value.deadline {
        time(b, words, BudgetPhrase::Deadline, at);
    }
    b.close();
}
/// Render one exact ledger, retaining its own source/parent period identities.
pub fn ledger(b: &mut Builder, value: &PoolLedger, words: &Words) {
    b.open(Role::Region);
    let label =
        words.book.say(words.language, BudgetPhrase::Source.id(), &[]).expect("checked ledger region label").text;
    b.name(&label);
    numbers(b, &value.numbers, value.left, words);
    text(b, words, BudgetPhrase::Source, &[]);
    proposed_task::funder(b, value.source, words);
    if let Some(parent) = value.parent {
        text(b, words, BudgetPhrase::Parent, &[]);
        proposed_task::funder(b, parent, words);
    }
    text(b, words, if value.closed { BudgetPhrase::Closed } else { BudgetPhrase::Open }, &[]);
    if let Some(at) = value.began {
        time(b, words, BudgetPhrase::Began, at);
    }
    if let Some(at) = value.next {
        time(b, words, BudgetPhrase::Next, at);
    }
    b.close();
}
