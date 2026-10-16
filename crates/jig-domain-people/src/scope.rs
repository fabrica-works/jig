//! The scope of an application request, subject or read.

/// Role scope named by an application's client (`client/domain/contract.md`, 7.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ApplicationScope {
    /// A party must hold a role in this project.
    Project(u32),
    /// A party must hold a deployment role.
    Deployment,
    /// The signed-in party's values across projects, without a role check.
    Own,
}

impl ApplicationScope {
    /// The people child's project identity for this scope.
    #[must_use]
    pub fn project(self) -> u32 {
        match self {
            Self::Project(project) => project,
            Self::Deployment => 0,
            Self::Own => unreachable!("own scope has no project"),
        }
    }
}
