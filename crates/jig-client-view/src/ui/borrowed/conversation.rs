use super::Borrowed;
use jig_client::thread::{LiveTurn, Thread};

use jig_client::{
    Latest,
    contract::{Delivery, LiveCall, ObjectKey, Part, RunState},
};
use skein_lib::List;
#[derive(Debug)]
pub struct Activity<'a> {
    pub state: &'a Thread,
    pub live: List<LiveTurn>,
    pub current_run: Option<&'a RunState>,
}
impl core::ops::Deref for Activity<'_> {
    type Target = Thread;
    fn deref(&self) -> &Thread {
        self.state
    }
}
impl Activity<'_> {
    #[must_use]
    pub fn names(&self, capacity: u32) -> List<ObjectKey> {
        self.state.names(capacity, self.live.as_slice())
    }
    #[must_use]
    pub fn parts(&self, capacity: u32) -> List<Part> {
        self.state.parts(capacity, self.live.as_slice())
    }
    #[must_use]
    pub fn shows(&self, key: ObjectKey) -> bool {
        self.state.shows(key, self.live.as_slice())
    }
}
impl<'a> Borrowed<'a> {
    #[must_use]
    pub fn conversation(&self) -> Activity<'a> {
        let mut state =
            Activity { state: &self.ui.conversation, live: self.ui.conversation.live.clone(), current_run: None };
        if self.ui.conversation.run_finished && self.ui.conversation.loaded >= self.ui.conversation.committed {
            return state;
        }
        for (handle, held) in &self.ui.handles.watches {
            let jig_client::contract::Subject::Run { task, .. } = held.subject else {
                continue;
            };
            if task != state.task {
                continue;
            }
            if let Some(document) = self.client.document(*handle)
                && let Some(Latest::Jig(Delivery::RunState(run))) = document.latest
            {
                state.current_run = Some(run);
            }
        }
        if state.current_run.is_some() {
            for at in 0..state.live.len() {
                let live = state.live.get_mut(at).expect("bounded projection turn exists");
                if let Some(LiveCall::Raw(_)) = live.call {
                    live.call = None;
                }
            }
        }
        state
    }
}
