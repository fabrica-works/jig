use super::World;
use jig_domain_tasks::Operation;
use skein_lib::List;
impl World {
    pub(super) fn remember_compound(&mut self, person: u64, period: u64, operations: &List<Operation>) {
        for operation in operations {
            match operation {
                Operation::Move { task, .. } => self.accounting_referee.moving(*task, person, period),
                Operation::Cancel { message: Some(number), .. }
                | Operation::Release { message: Some(number), .. }
                | Operation::Amend { message: number, .. } => self.message = self.message.max(*number),
                Operation::Message { word, .. } => self.message = self.message.max(word.number),
                Operation::Stop { .. }
                | Operation::Cancel { message: None, .. }
                | Operation::Release { message: None, .. } => {}
            }
        }
    }
}
