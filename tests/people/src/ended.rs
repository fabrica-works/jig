//! Scripted task-owner joins for ended positions in the people world.
use super::{Event, Key, Stored, Token, World, people};
impl World {
    pub fn end_conversation(&mut self, party: u64, task: u64, last_reply: u64) {
        self.ended_chats.insert(task, (party, last_reply));
        self.send(Event::ChatState {
            party,
            project: 1,
            task,
            last_reply,
            live: false,
            seen: None,
            replies: Box::new([]),
        });
    }
    pub(super) fn restore_ended(&mut self) {
        for (task, (party, last_reply)) in self.ended_chats.clone() {
            let Some(Stored::EndedRead { through, .. }) = self.records.get(&Key::EndedRead { task, party }) else {
                panic!("ended owner has a position")
            };
            self.send(Event::ChatState {
                party,
                project: 1,
                task,
                last_reply,
                live: false,
                seen: Some(*through),
                replies: Box::new([]),
            });
        }
    }
    pub(super) fn mark_ended(&mut self, request: Token, party: u64, task: u64) -> bool {
        let Some((owner, last_reply)) = self.ended_chats.get(&task) else { return false };
        assert_eq!(*owner, party);
        let Some(Stored::EndedRead { through, .. }) = self.records.get(&Key::EndedRead { task, party }) else {
            panic!("ended position")
        };
        self.domain.mark_ended(
            &self.env,
            people::EndedMark { request, project: 1, last_reply: *last_reply, seen: *through },
            &mut self.out,
        );
        true
    }
}
