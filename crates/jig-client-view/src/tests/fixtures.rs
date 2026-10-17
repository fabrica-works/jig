use crate::records::{RECORDS_VERSION, Record};
use jig_client_domain::contract::Text;
use jig_domain_client::address::Address;
pub const PARTY: u64 = u64::MAX;
pub const TASK: u64 = u64::MAX;
pub fn place() -> Record {
    Record::Place { version: RECORDS_VERSION, party: PARTY, address: Address::Deployment }
}
pub fn draft() -> Record {
    Record::Draft {
        version: RECORDS_VERSION,
        party: PARTY,
        task: TASK,
        words: Text::new(Box::from(b"words".as_slice()), 65536).unwrap(),
        touched: u64::MAX,
    }
}
pub fn places() -> skein_lib::List<Record> {
    use jig_domain_client::address::{Address, ProjectSection, TaskSection};
    let mut addresses = skein_lib::List::with_capacity(21);
    for address in [
        Address::Inbox { project: None },
        Address::Inbox { project: Some(u32::MAX) },
        Address::Conversations { project: None },
        Address::Conversations { project: Some(u32::MAX) },
        Address::Deployment,
    ] {
        addresses.push(address).unwrap();
    }
    for section in [
        TaskSection::Overview,
        TaskSection::Activity,
        TaskSection::Plan,
        TaskSection::History,
        TaskSection::Budget,
        TaskSection::Panels,
    ] {
        addresses.push(Address::Task { task: u64::MAX, section }).unwrap();
    }
    for section in [
        ProjectSection::Overview,
        ProjectSection::Goals,
        ProjectSection::Tasks,
        ProjectSection::Notes,
        ProjectSection::Settings,
        ProjectSection::Resources,
        ProjectSection::Policy,
        ProjectSection::Roles,
        ProjectSection::Spend,
        ProjectSection::Recurring,
    ] {
        addresses.push(Address::Project { project: u32::MAX, section }).unwrap();
    }
    let mut saved = skein_lib::List::with_capacity(addresses.len());
    for address in addresses.into_boxed() {
        saved.push(Record::Place { version: RECORDS_VERSION, party: u64::MAX, address }).unwrap();
    }
    saved
}
