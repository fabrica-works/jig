//! Family meanings are rebuilt with each tree; no token is reused (view.md, 5).
use jig_client::contract::{ObjectKey, Revision};
use jig_client_view::{Act, Binding, Owner};
use jig_test_client_domain::first;
use skein_lib::{List, Token};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Meaning {
    Action(first::Act),
    Parameters { object: Token, revision: Revision },
    Object(ObjectKey),
}
#[derive(Debug)]
struct Entry {
    token: Token,
    meaning: Meaning,
}
#[derive(Debug)]
pub struct Bindings {
    entries: List<Entry>,
    next: u64,
    text: u32,
}
impl Bindings {
    #[must_use]
    pub fn new(capacity: u32, text: u32) -> Self {
        Self { entries: List::with_capacity(capacity), next: 0, text }
    }
    pub fn begin(&mut self) {
        self.entries.clear();
    }
    pub fn insert(&mut self, meaning: Meaning) -> Binding {
        self.next = self.next.checked_add(1).expect("family tokens fit");
        let token = Token::new(self.next);
        self.entries.push(Entry { token, meaning }).expect("family meanings fit");
        Binding { owner: Owner::Side { connector: jig_test_client_domain::FIRST }, token }
    }
    #[must_use]
    pub fn object(&self, token: Token) -> Option<ObjectKey> {
        for entry in &self.entries {
            if entry.token == token {
                return match &entry.meaning {
                    Meaning::Object(key) => Some(*key),
                    Meaning::Action(_) | Meaning::Parameters { .. } => None,
                };
            }
        }
        None
    }
}
#[must_use]
pub fn decode(bindings: &Bindings, token: Token, act: Act) -> Option<first::Act> {
    for entry in &bindings.entries {
        if entry.token == token {
            return match &entry.meaning {
                Meaning::Action(action) => match act {
                    Act::Press | Act::Submit | Act::Toggle => Some(action.clone()),
                    Act::Input { .. } => None,
                },
                Meaning::Parameters { object, revision } => match act {
                    Act::Input { value } => {
                        if value.len() > usize::try_from(bindings.text).expect("text fits usize") {
                            return None;
                        }
                        Some(first::Act::Parameters {
                            object: *object,
                            revision: *revision,
                            value: jig_client_view::number(&value)?,
                        })
                    }
                    Act::Press | Act::Submit | Act::Toggle => None,
                },
                Meaning::Object(_) => None,
            };
        }
    }
    None
}
#[must_use]
pub fn worst_case(capacity: u32) -> Option<u64> {
    List::<Entry>::worst_case(capacity)
}
