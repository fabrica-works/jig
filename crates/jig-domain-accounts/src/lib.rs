//! Accounts and their sources, component grants and usability (domain/accounts.md, section 3;
//! domain/engine.md, section 12). Each account is the deployment's or a party's, numbered by a
//! counter that never gives a number twice, with its sources in its holder's order; each source
//! keeps its own state. The component owns refreshing, records and generation numbering. This
//! child keeps names, validity, limits, refusals and failures, and releases all wants at close.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod boundary;
mod checks;
mod domain;
mod limits;
mod providers;
mod records;
mod restore;
mod signing;
pub use boundary::{
    ADDRESS, Answer, End, Ended, Event, Fact, Failure, FromComponent, Grant, GrantTable, Pending, Place, Request,
    Signing, State, ToComponent, Turned, Unread,
};
pub use checks::{Bound, Refusal, SignIn, room_for_sign_in};
pub use domain::{Account, Domain, fire, max_out, step};
pub use limits::{Limits, worst_case};
pub use providers::{Offers, Provider, Providers};
pub use records::{Configured, DISPLAY, Held, Holder, Key, Kind, NAME, SUBJECT, Stored};
