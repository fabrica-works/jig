use super::*;
pub fn family_request(record: Box<[u8]>) -> Box<[u8]> {
    let value =
        w::FamilyRequest::new(&wire(), w::FamilyRequestParts { scope: w::Scope::Own, record }).expect("family request");
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("wire size"));
    value.encode(&mut writer).expect("measured family request");
    writer.finish()
}
pub fn family_record(record: Box<[u8]>) -> Box<[u8]> {
    let window = w::Window::new(&wire(), w::WindowParts { rows: 3, after: None }).expect("window");
    let value = w::FamilyRecord::new(&wire(), w::FamilyRecordParts { scope: w::Scope::Own, window, record })
        .expect("family record");
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("wire size"));
    value.encode(&mut writer).expect("measured family record");
    writer.finish()
}

pub fn not_served(owner: w::Owner) -> w::NotServed {
    let lim = wire();
    let family = w::Owner::Family(w::Family::new(&lim, w::FamilyParts { number: 9 }).expect("family"));
    let mut served = List::with_capacity(2);
    for (owner, first, last) in [(w::Owner::Jig, 13, 13), (family, 2, 2)] {
        served.push(w::Served::new(&lim, w::ServedParts { owner, first, last }).expect("range")).expect("range slot");
    }
    w::NotServed::new(&lim, w::NotServedParts { owner, served }).expect("refusal")
}
