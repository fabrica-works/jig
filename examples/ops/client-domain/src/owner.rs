//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides, family addresses and application vocabulary.
use skein_lib::Token;
/// The table echoes the requesting child's token, or a restored record's scope.
/// The child request token, or restored request scope with no live child.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    /// jig owns this live request.
    Jig(Token),
    /// The observability side owns this token.
    Observability(Token),
    /// Infrastructure owns this live request.
    Infrastructure(Token),
    /// The original child disappeared on reload.
    Kept { scope: u64 },
}
