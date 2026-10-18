//! Application-named browser-shell stories (protocol/browser.md, 15).
use skein_fake_person::Find;

pub mod attacks;
pub mod pages;
pub mod stories;

/// What an application's tier names of its pages for jig's stories.
pub struct States {
    /// What its page shows once it holds the engine's state and its stream is live.
    pub live: Find,
    /// What it shows while the engine is out of reach.
    pub offline: Find,
    /// What it shows signed out: its sign-in page's words.
    pub signed_out: Find,
}

/// What an attack needs of the application's mapping and visible result.
pub struct Target<'a> {
    /// The prefix at which the server mounts jig's mapping.
    pub prefix: &'a [u8],
    /// A page opened after the attack, or framed by another origin.
    pub page: &'a [u8],
    /// A keyed request's record, as the application's client would send it.
    pub request: &'a [u8],
    /// What the page shows after the request is made.
    pub done: Find,
}

#[cfg(test)]
mod tests;
