//! Owner tokens carried unchanged by the tree (client/domain/view.md, section 5).

use crate::Owner;
use skein_lib::Token;

/// A meaning kept by its owner's view for the last built tree.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Binding {
    pub owner: Owner,
    pub token: Token,
}
