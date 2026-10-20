//! Total vocabulary translation (protocol/llm.md, section 5.2).
use crate::table;
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use skein_lib::{Secret, Time};
use skein_oauth as oauth;
use skein_oauth_accounts as driver;

/// A grant's value travels beside the child's secret-free event; the
/// driver's tokens are bearer credentials. A record to keep goes to the
/// owner's store, never to the child.
#[derive(PartialEq, Eq, Debug)]
pub enum Up {
    Granted {
        event: accounts::FromComponent,
        account: u32,
        generation: u64,
        value: table::Value,
        expires: Time,
    },
    Child(accounts::FromComponent),
    Visited {
        source: u32,
        address: Box<[u8]>,
    },
    SignedIn {
        source: u32,
        subject: Box<[u8]>,
        display: Box<[u8]>,
    },
    SignInFailed {
        source: u32,
        failure: accounts::Failure,
    },
    SignedOut {
        source: u32,
        confirmed: bool,
    },
    Removed {
        source: u32,
    },
    /// A renewed record for the secrets' store; nothing new is lent until
    /// the store's answer goes back as `Kept` (llm.md 5.2).
    Keep {
        account: u32,
        record: Candidate,
    },
}

/// A record on its way to the store; printed, it shows only its generation.
#[derive(PartialEq, Eq)]
pub struct Candidate(pub oauth::SavedToken);

impl core::fmt::Debug for Candidate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Candidate").field("generation", &self.0.generation).finish_non_exhaustive()
    }
}

#[must_use]
pub fn down(request: accounts::ToComponent) -> driver::Request {
    match request {
        accounts::ToComponent::Want { account } => driver::Request::Grant { account },
        accounts::ToComponent::Rejected { account, generation } => driver::Request::Rejected { account, generation },
        accounts::ToComponent::Release { account } => driver::Request::Release { account },
        accounts::ToComponent::Close => driver::Request::Close,
    }
}

#[must_use]
pub fn up(event: driver::Event, now: Time, required: &[u8]) -> Up {
    match event {
        driver::Event::Granted { account, token, generation, valid } => Up::Granted {
            event: accounts::FromComponent::Granted { form: form(&table::Scheme::Bearer), account, generation, valid },
            account,
            generation,
            value: table::Value { token: Secret::new(token), scheme: table::Scheme::Bearer },
            expires: now.saturating_add(valid),
        },
        driver::Event::Expiring { account, generation } => {
            Up::Child(accounts::FromComponent::Expiring { account, generation })
        }
        driver::Event::Failed { account, ends, failure } => match ends {
            driver::Ends::Grant | driver::Ends::Refresh => failed(account, classify(failure)),
            driver::Ends::SignIn => Up::SignInFailed { source: account, failure: classify(failure) },
            driver::Ends::SignOut => Up::SignedOut { source: account, confirmed: false },
        },
        driver::Event::Refused { account, asked, why } => match asked {
            driver::Asked::Grant | driver::Asked::Want => failed(account, refused(why)),
            driver::Asked::SignIn | driver::Asked::Add | driver::Asked::Redirected => {
                Up::SignInFailed { source: account, failure: refused(why) }
            }
            driver::Asked::SignOut | driver::Asked::Remove => Up::SignedOut { source: account, confirmed: false },
            driver::Asked::HandIn => unreachable!("the layer checks each handed-in record before handing it in"),
        },
        driver::Event::Closed => Up::Child(accounts::FromComponent::Closed),
        driver::Event::Visit { account, url } => {
            if url.len() > usize::try_from(accounts::ADDRESS).expect("address bound") {
                Up::SignInFailed { source: account, failure: accounts::Failure::Misconfigured }
            } else {
                Up::Visited { source: account, address: url }
            }
        }
        driver::Event::SignedIn { account, generation: _, missing, subject } => {
            let Some(subject) = subject else {
                return Up::SignInFailed { source: account, failure: accounts::Failure::Misconfigured };
            };
            if !missing.is_empty() {
                if let Some(scope) = missing_index(required, &missing) {
                    return Up::SignInFailed { source: account, failure: accounts::Failure::MissingScope { scope } };
                }
                return Up::SignInFailed { source: account, failure: accounts::Failure::Misconfigured };
            }
            Up::SignedIn { source: account, subject, display: Box::default() }
        }
        driver::Event::Keep { account, record } => Up::Keep { account, record: Candidate(record) },
        driver::Event::Lock { .. } | driver::Event::Unlock { .. } => {
            unreachable!("no record is shared: each source's record has one keeper, the engine's store")
        }
        driver::Event::Removed { account } => Up::Removed { source: account },
        driver::Event::SignedOut { account, revoked } => Up::SignedOut { source: account, confirmed: revoked },
    }
}

fn failed(account: u32, failure: accounts::Failure) -> Up {
    Up::Child(accounts::FromComponent::Failed { account, failure })
}

fn classify(failure: driver::Failure) -> accounts::Failure {
    match failure {
        driver::Failure::Busy | driver::Failure::NotKept => accounts::Failure::Later { after: None },
        driver::Failure::Exchange(failure) => exchange(failure),
        driver::Failure::NotGranted | driver::Failure::AnotherSubject | driver::Failure::Expired => {
            accounts::Failure::SignInAgain
        }
        driver::Failure::Unloaded { at, why } => accounts::Failure::Unloaded { at: place(at), why: unread(why) },
    }
}

fn exchange(failure: oauth::Failure) -> accounts::Failure {
    match failure {
        oauth::Failure::RateLimited { retry_after } => accounts::Failure::Later { after: Some(retry_after) },
        oauth::Failure::Unavailable | oauth::Failure::TimedOut | oauth::Failure::Malformed | oauth::Failure::Limit => {
            accounts::Failure::Later { after: None }
        }
        oauth::Failure::Refused => accounts::Failure::SignInAgain,
        oauth::Failure::ClientRejected => accounts::Failure::Misconfigured,
        oauth::Failure::Cancelled => accounts::Failure::Cancelled,
        oauth::Failure::Incomplete | oauth::Failure::Unverified { .. } | oauth::Failure::InvalidRedirect => {
            accounts::Failure::Misconfigured
        }
    }
}

fn refused(why: driver::Refusal) -> accounts::Failure {
    match why {
        driver::Refusal::NoRecord => accounts::Failure::SignInAgain,
        driver::Refusal::Closed => accounts::Failure::Cancelled,
        driver::Refusal::Full { bound: _ } | driver::Refusal::Busy => accounts::Failure::Later { after: None },
        driver::Refusal::Taken
        | driver::Refusal::Unusable(_)
        | driver::Refusal::NotWaiting
        | driver::Refusal::Redirect
        | driver::Refusal::Account
        | driver::Refusal::Source
        | driver::Refusal::NotAccessOnly
        | driver::Refusal::Held
        | driver::Refusal::Record(_) => accounts::Failure::Misconfigured,
    }
}

fn place(at: driver::Place) -> accounts::Place {
    match at {
        driver::Place::Directory => accounts::Place::Directory,
        driver::Place::File => accounts::Place::File,
    }
}

fn unread(why: driver::Unloaded) -> accounts::Unread {
    match why {
        driver::Unloaded::Unreadable => accounts::Unread::Unreadable,
        driver::Unloaded::Refused(_) => accounts::Unread::Refused,
        driver::Unloaded::Failed(_) => accounts::Unread::Failed,
        driver::Unloaded::Stalled => accounts::Unread::Stalled,
        driver::Unloaded::TooLarge { size: _ } => accounts::Unread::TooLarge,
    }
}

/// The secret-free form announced for the same scheme held in the table.
#[must_use]
pub const fn form(scheme: &table::Scheme) -> accounts::Form {
    match scheme {
        table::Scheme::Bearer => accounts::Form::Credential(accounts::Scheme::Bearer),
        table::Scheme::Key => accounts::Form::Credential(accounts::Scheme::Key),
        table::Scheme::Withdrawn { .. } => accounts::Form::Withdrawn,
    }
}
/// A child's LLM account sign-in asks for an add before its driver request.
#[derive(PartialEq, Eq, Debug)]
pub enum Down {
    SignIn { source: u32, provider: u32 },
    SignOut { source: u32 },
}
#[must_use]
pub fn down_signing(signing: accounts::Signing) -> Down {
    match signing {
        accounts::Signing::Keep { .. }
        | accounts::Signing::Forget { .. }
        | accounts::Signing::Restore { .. }
        | accounts::Signing::Restored => {
            unreachable!("the host holds the LLM-key and restart paths until their component is wired")
        }
        accounts::Signing::SignIn { source, provider } => Down::SignIn { source, provider },
        accounts::Signing::SignOut { source } | accounts::Signing::Replaced { source, by: _ } => {
            Down::SignOut { source }
        }
    }
}
fn missing_index(required: &[u8], missing: &[u8]) -> Option<u32> {
    let mut start = 0;
    let mut index = 0;
    for end in 0..=required.len() {
        if end == required.len() || required.get(end) == Some(&b' ') {
            let scope = required.get(start..end).expect("scope within required bytes");
            let mut from = 0;
            for to in 0..=missing.len() {
                if to == missing.len() || missing.get(to) == Some(&b' ') {
                    if !scope.is_empty() && scope == missing.get(from..to).expect("scope within missing bytes") {
                        return Some(index);
                    }
                    from = to.checked_add(1).expect("bounded scope bytes");
                }
            }
            start = end.checked_add(1).expect("bounded scope bytes");
            index = index.checked_add(1).expect("bounded scope count");
        }
    }
    None
}
