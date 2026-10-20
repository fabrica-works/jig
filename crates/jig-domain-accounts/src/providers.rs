//! The configured LLM providers whose accounts this child holds (accounts.md, 3.1 and 3.4;
//! configuration.md, 3). Numbered apart from identity providers (people.md, 3).
use alloc::boxed::Box;

/// Every configured LLM provider; empty when the configuration names none.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Providers {
    pub providers: Box<[Provider]>,
}

impl Providers {
    /// No provider configured.
    #[must_use]
    pub fn none() -> Self {
        Self { providers: Box::new([]) }
    }
    /// The provider of this number.
    #[must_use]
    #[expect(clippy::manual_find, reason = "the strict subset iterates without a closure")]
    pub fn get(&self, number: u32) -> Option<&Provider> {
        for provider in &self.providers {
            if provider.number == number {
                return Some(provider);
            }
        }
        None
    }
    /// The provider serving a configured model, by its name.
    #[must_use]
    pub fn serving(&self, model: &[u8]) -> Option<&Provider> {
        for provider in &self.providers {
            for name in &provider.models {
                if **name == *model {
                    return Some(provider);
                }
            }
        }
        None
    }
}

/// One LLM provider: its number, the configured models it serves and the sources it offers.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Provider {
    pub number: u32,
    /// Configured model names; each configured model is in at most one provider.
    pub models: Box<[Box<[u8]>]>,
    pub offers: Offers,
}

/// The sources a provider offers its accounts. Offering none of them is the `none` source:
/// nothing held, as for a local model (accounts.md, 3.4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Offers {
    /// A sign-in, with the provider's public registration, kept by configuration for the driver.
    pub sign_in: bool,
    pub key: bool,
    /// Program names, such as `claude_code` for Claude Code.
    pub programs: Box<[Box<[u8]>]>,
}

impl Offers {
    /// Offering no source: the provider's one account is the deployment's `none` account.
    #[must_use]
    pub fn none(&self) -> bool {
        !self.sign_in && !self.key && self.programs.is_empty()
    }
}
