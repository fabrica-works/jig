//! Account failure details, translated from the component (engine.md, 12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountPhrase {
    Later,
    LaterUnknown,
    Unloaded,
    Cancelled,
    Directory,
    File,
    Unreadable,
    Refused,
    Failed,
    Stalled,
    TooLarge,
    Provider,
    HeldByDeployment,
    HeldByParty,
    Plan,
    Sources,
    Source,
    SignIn,
    Key,
    Program,
    None,
    DefaultUses,
    PaysParty,
    PaysProject,
    ManageUsage,
    Catalogue,
    KeyBytes,
    Ready,
    Unavailable,
    UnknownReadiness,
    Settled,
    Pending,
    UnknownSettlement,
    OffersCut,
    ReadinessCut,
    SignInPlan,
    RefusalNotSignedIn,
    RefusalNotOurs,
    RefusalCancelled,
    RefusalMoving,
}
impl AccountPhrase {
    pub(super) const fn hole(self) -> Option<skein_ui::phrases::HoleKind> {
        use skein_ui::phrases::HoleKind;
        match self {
            Self::Later => Some(HoleKind::Wall),
            Self::KeyBytes
            | Self::Provider
            | Self::HeldByParty
            | Self::Source
            | Self::PaysParty
            | Self::PaysProject => Some(HoleKind::Number),
            Self::SignInPlan
            | Self::RefusalNotSignedIn
            | Self::RefusalNotOurs
            | Self::RefusalCancelled
            | Self::RefusalMoving
            | Self::Catalogue
            | Self::Ready
            | Self::Unavailable
            | Self::UnknownReadiness
            | Self::Settled
            | Self::Pending
            | Self::UnknownSettlement
            | Self::OffersCut
            | Self::ReadinessCut
            | Self::ManageUsage
            | Self::LaterUnknown
            | Self::Unloaded
            | Self::Cancelled
            | Self::Directory
            | Self::File
            | Self::Unreadable
            | Self::Refused
            | Self::Failed
            | Self::Stalled
            | Self::TooLarge
            | Self::HeldByDeployment
            | Self::Plan
            | Self::Sources
            | Self::SignIn
            | Self::Key
            | Self::Program
            | Self::None
            | Self::DefaultUses => None,
        }
    }
    pub(super) const fn number(self) -> u16 {
        match self {
            Self::Later => 9400,
            Self::LaterUnknown => 9421,
            Self::Unloaded => 9401,
            Self::Cancelled => 9402,
            Self::Directory => 9403,
            Self::File => 9404,
            Self::Unreadable => 9405,
            Self::Refused => 9406,
            Self::Failed => 9407,
            Self::Stalled => 9408,
            Self::TooLarge => 9409,
            Self::Provider => 9452,
            Self::HeldByDeployment => 9453,
            Self::HeldByParty => 9454,
            Self::Plan => 9455,
            Self::Sources => 9456,
            Self::Source => 9457,
            Self::SignIn => 9458,
            Self::Key => 9459,
            Self::Program => 9460,
            Self::None => 9461,
            Self::DefaultUses => 9462,
            Self::PaysParty => 9463,
            Self::PaysProject => 9464,
            Self::ManageUsage => 9473,
            Self::Catalogue => 9474,
            Self::KeyBytes => 9475,
            Self::Ready => 9476,
            Self::Unavailable => 9477,
            Self::UnknownReadiness => 9478,
            Self::Settled => 9479,
            Self::Pending => 9480,
            Self::UnknownSettlement => 9481,
            Self::OffersCut => 9482,
            Self::ReadinessCut => 9483,
            Self::SignInPlan => 9484,
            Self::RefusalNotSignedIn => 9486,
            Self::RefusalNotOurs => 9487,
            Self::RefusalCancelled => 9488,
            Self::RefusalMoving => 9489,
        }
    }
    pub(super) const fn label(self) -> &'static [u8] {
        match self {
            Self::Later => b"Asks again at ",
            Self::LaterUnknown => b"Asks again later",
            Self::Unloaded => b"Account State Unloaded",
            Self::Cancelled => b"Account State Cancelled",
            Self::Directory => b"Record directory",
            Self::File => b"Record file",
            Self::Unreadable => b"Record unreadable",
            Self::Refused => b"Record refused",
            Self::Failed => b"Record failed",
            Self::Stalled => b"Record stalled",
            Self::TooLarge => b"Record too large",
            Self::ManageUsage => b"Manage usage",
            Self::Catalogue => b"LLM provider offers",
            Self::KeyBytes => b"Maximum key bytes ",
            Self::Ready => b"Ready for this payer",
            Self::Unavailable => b"No usable account for this payer",
            Self::UnknownReadiness => b"Readiness unknown",
            Self::Settled => b"Account requests settled",
            Self::Pending => b"Account request pending",
            Self::UnknownSettlement => b"Program checks unknown",
            Self::OffersCut => b"More provider offers omitted",
            Self::ReadinessCut => b"More payer readiness omitted",
            Self::SignInPlan => b"Sign-ins use plan terms",
            Self::RefusalNotSignedIn => b"Account move: not signed in",
            Self::RefusalNotOurs => b"Account move: not ours",
            Self::RefusalCancelled => b"Account move: cancelled",
            Self::RefusalMoving => b"Account move: moving",

            Self::Provider => b"LLM provider ",
            Self::HeldByDeployment => b"Held by the deployment",
            Self::HeldByParty => b"Held by party ",
            Self::Plan => b"A plan's account",
            Self::Sources => b"Sources",
            Self::Source => b"Source ",
            Self::SignIn => b"Sign-in",
            Self::Key => b"Key",
            Self::Program => b"Program",
            Self::None => b"No credential",
            Self::DefaultUses => b"Default uses",
            Self::PaysParty => b"Pays for party ",
            Self::PaysProject => b"Pays for project ",
        }
    }
}
