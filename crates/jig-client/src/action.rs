//! Acts decoded by the view, owned by this domain; client/domain/client.md, 4.1.
//! Jig 21 adds object act parameters and confirmations; jig 22 adds page filters.
use crate::{
    address::Address,
    contract::{ActKind, InboxGroup, Text},
    fields::Purpose,
};
use skein_lib::Token;
/// A person action already decoded from the semantic tree.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Act {
    /// Go to a jig page.
    Go { address: Address },
    /// Edit a field.
    Edit { field: Purpose, text: Text },
    /// Submit a composer or form.
    Submit { field: Purpose },
    /// Open a confirmation for an object act.
    Intend { object: Token, act: ActKind },
    /// Choose an option in the confirmation.
    Choose { option: u16 },
    /// Confirm the open act.
    Confirm,
    /// Dismiss the confirmation or notice.
    Dismiss,
    /// Open or fold an object.
    Fold { object: Token, open: bool },
    /// Filter an inbox group.
    Filter { group: InboxGroup, project: Option<u32> },
    /// Read more at a window end.
    More { window: Token, end: End },
    /// Choose a sign in provider.
    SignIn { provider: u16 },
}

/// The end of a window requested by a person.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum End {
    /// Earlier rows.
    Before,
    /// Later rows.
    After,
}
