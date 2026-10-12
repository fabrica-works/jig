//! Total vocabulary translation (protocol/llm.md, section 5.2).
use jig_domain_accounts as accounts;
use skein_lib::{Secret, Time};
use skein_oauth as oauth;
use skein_oauth_accounts as driver;

/// A grant's value travels beside the child's secret-free event.
#[derive(PartialEq, Eq, Debug)]
pub enum Up {
    Granted { event: accounts::FromComponent, account: u32, generation: u64, value: Secret, expires: Time },
    Child(accounts::FromComponent),
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
pub fn up(event: driver::Event, now: Time) -> Up {
    match event {
        driver::Event::Granted { account, token, generation, valid } => Up::Granted {
            event: accounts::FromComponent::Granted { account, generation, valid },
            account,
            generation,
            value: Secret::new(token),
            expires: now.saturating_add(valid),
        },
        driver::Event::Expiring { account, generation } => {
            Up::Child(accounts::FromComponent::Expiring { account, generation })
        }
        driver::Event::Failed { account, ends, failure } => {
            match ends {
                driver::Ends::Grant => (),
                driver::Ends::SignIn => unreachable!("the engine runs no sign-in"),
            }
            failed(account, classify(failure))
        }
        driver::Event::Refused { account, asked, why } => {
            match asked {
                driver::Asked::Grant => (),
                driver::Asked::SignIn | driver::Asked::Redirected => unreachable!("the engine runs no sign-in"),
                driver::Asked::HandIn => unreachable!("the layer checks each handed-in record before handing it in"),
            }
            failed(account, refused(why))
        }
        driver::Event::Closed => Up::Child(accounts::FromComponent::Closed),
        driver::Event::Visit { account: _, url: _ } | driver::Event::SignedIn { account: _, generation: _ } => {
            unreachable!("the engine runs no sign-in")
        }
        driver::Event::Keep { account: _, record: _ } => unreachable!("the engine's accounts use the private keeper"),
    }
}

fn failed(account: u32, failure: accounts::Failure) -> Up {
    Up::Child(accounts::FromComponent::Failed { account, failure })
}

fn classify(failure: driver::Failure) -> accounts::Failure {
    match failure {
        driver::Failure::NotKept => accounts::Failure::Later { after: None },
        driver::Failure::Exchange(failure) => exchange(failure),
        driver::Failure::Expired => accounts::Failure::SignInAgain,
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
        oauth::Failure::InvalidRedirect => unreachable!("the engine runs no sign-in or redirect exchange"),
    }
}

fn refused(why: driver::Refusal) -> accounts::Failure {
    match why {
        driver::Refusal::Closed => accounts::Failure::Cancelled,
        driver::Refusal::Full { bound: _ } => accounts::Failure::Later { after: None },
        driver::Refusal::Busy
        | driver::Refusal::NotWaiting
        | driver::Refusal::Redirect
        | driver::Refusal::Account
        | driver::Refusal::Source
        | driver::Refusal::NotAccessOnly
        | driver::Refusal::Held
        | driver::Refusal::Record(_) => {
            unreachable!("the child wants each configured account once; records are checked before handing in")
        }
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
