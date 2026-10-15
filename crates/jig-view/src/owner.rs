//! jig's owner mapping onto the opaque tree owner (client/domain/view.md, section 3).

use crate::ui::tree;

/// The view child that interprets a token, key, style, region, picture or phrase.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Owner {
    /// jig's own view.
    Jig,
    /// One of the application's owners, by the number both its roots share.
    Application { number: u16 },
}

impl Owner {
    #[must_use]
    pub const fn opaque(self) -> tree::Owner {
        let number = match self {
            Self::Jig => 65_536,
            Self::Application { number } => {
                let [low, high] = number.to_le_bytes();
                u32::from_le_bytes([low, high, 0, 0])
            }
        };
        tree::Owner { number }
    }

    #[must_use]
    pub const fn from_opaque(owner: tree::Owner) -> Option<Self> {
        match owner.number {
            65_536 => Some(Self::Jig),
            0..=65_535 => {
                let [low, high, _, _] = owner.number.to_le_bytes();
                Some(Self::Application { number: u16::from_le_bytes([low, high]) })
            }
            65_537..=u32::MAX => None,
        }
    }
}
