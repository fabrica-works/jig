//! Jig's closed style vocabulary (client/domain/view.md, 13).
use crate::{Owner, Style};
/// Appearance hints; roles, states and words carry all facts independently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JigStyle {
    Card,
    Chip,
    Muted,
    Warning,
    Danger,
    Success,
    ButtonWeight,
    DenseList,
}
impl JigStyle {
    /// Stable token interpreted by jig's shells and restylable by applications.
    #[must_use]
    pub const fn token(self) -> Style {
        let token = match self {
            Self::Card => 0,
            Self::Chip => 1,
            Self::Muted => 2,
            Self::Warning => 3,
            Self::Danger => 4,
            Self::Success => 5,
            Self::ButtonWeight => 6,
            Self::DenseList => 7,
        };
        Style { owner: Owner::Jig.opaque(), token }
    }
}
