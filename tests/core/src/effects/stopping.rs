use super::World;
use jig_test_domain as root;

impl World {
    pub fn stop(&mut self) {
        self.send(root::Event::Stop);
    }
    #[must_use]
    pub fn task(&self) -> u64 {
        self.assigned.as_ref().expect("assigned chat").task
    }
    #[must_use]
    pub fn attempt(&self) -> u64 {
        self.assigned.as_ref().expect("assigned chat").attempt
    }
}
