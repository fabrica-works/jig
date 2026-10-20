//! The checks a sign-in and a new source make (accounts.md, 3.3 and 3.6), as functions of the
//! child's state alone; a party's sign-in makes them (`signing`), and adding keys will.
use crate::records::within;
use crate::{Domain, Holder, Kind, Limits, SUBJECT};

/// What a sign-in's subject does with its provider's accounts (accounts.md, 3.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SignIn {
    /// A subject no account of the provider holds: a new account, with its signed-in source.
    New,
    /// A subject its holder already holds: that account's record is renewed.
    Renew { account: u32 },
}

/// Why a sign-in or a new source is refused, naming what refuses it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Refusal {
    /// A bound of the deployment's (accounts.md, 3.6) would be passed.
    Limit { bound: Bound },
    /// Another holder holds this provider's account of the subject: the provider's rule that a
    /// subscription serves one person.
    Held { provider: u32 },
    /// No configured provider of this number offers this source.
    Provider { provider: u32 },
    /// A subject empty, or longer than `SUBJECT`.
    Subject,
    /// No account or source of this number, or none of the holder's.
    Unknown,
    /// The counter has no numbers left for it.
    Numbers,
    /// A source of a kind this request does not serve: a sign-out serves a signed-in source.
    Kind { kind: Kind },
    /// The child takes no request before its start, nor once closing.
    Closing,
}

/// The bounds of accounts.md, 3.6 that the child keeps.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Bound {
    Accounts,
    HolderAccounts,
    Sources,
    SignIns,
}

impl Domain {
    /// A sign-in's subject, for a holder at a provider: a new subject makes an account, the
    /// holder's own renews it, and another holder's is refused (accounts.md, 3.3).
    ///
    /// # Errors
    ///
    /// The refusal names the provider's rule, the provider, the subject or the bound passed.
    pub fn sign_in(&self, limits: &Limits, holder: Holder, provider: u32, subject: &[u8]) -> Result<SignIn, Refusal> {
        if subject.is_empty() || !within(subject, SUBJECT) {
            return Err(Refusal::Subject);
        }
        match self.providers.get(provider) {
            Some(offered) if offered.offers.sign_in => {}
            Some(_) | None => return Err(Refusal::Provider { provider }),
        }
        for (number, account) in &self.accounts {
            if account.provider != Some(provider) || *account.subject != *subject {
                continue;
            }
            if account.holder != holder {
                return Err(Refusal::Held { provider });
            }
            for (_, source) in &account.sources {
                if self.sources.get(source).expect("an account's sources are kept").kind == Kind::SignIn {
                    return Ok(SignIn::Renew { account: *number });
                }
            }
            self.room_for_source(limits, *number)?;
            return Ok(SignIn::Renew { account: *number });
        }
        self.room_for_account(limits, holder)?;
        Ok(SignIn::New)
    }
    /// Room for a holder's new account and its first source.
    ///
    /// # Errors
    ///
    /// The deployment's accounts, the holder's, or the counter's numbers.
    pub fn room_for_account(&self, limits: &Limits, holder: Holder) -> Result<(), Refusal> {
        if self.accounts.len() >= limits.accounts.min(self.accounts.capacity()) {
            return Err(Refusal::Limit { bound: Bound::Accounts });
        }
        let mut held: u32 = 0;
        for (_, account) in &self.accounts {
            if account.holder == holder {
                held = held.saturating_add(1);
            }
        }
        if held >= limits.holder_accounts {
            return Err(Refusal::Limit { bound: Bound::HolderAccounts });
        }
        if self.numbers_left() < 2 {
            return Err(Refusal::Numbers);
        }
        Ok(())
    }
    /// Room for one more source in an account.
    ///
    /// # Errors
    ///
    /// No such account, its sources' bound, or the counter's numbers.
    pub fn room_for_source(&self, limits: &Limits, account: u32) -> Result<(), Refusal> {
        let Some(entry) = self.accounts.get(&account) else { return Err(Refusal::Unknown) };
        if entry.sources.len() >= limits.sources.min(entry.sources.capacity()) {
            return Err(Refusal::Limit { bound: Bound::Sources });
        }
        if self.numbers_left() < 1 {
            return Err(Refusal::Numbers);
        }
        Ok(())
    }
}

/// Room for one more sign-in beside those under way.
///
/// # Errors
///
/// The sign-ins' bound.
pub const fn room_for_sign_in(limits: &Limits, under_way: u32) -> Result<(), Refusal> {
    if under_way >= limits.sign_ins {
        return Err(Refusal::Limit { bound: Bound::SignIns });
    }
    Ok(())
}
