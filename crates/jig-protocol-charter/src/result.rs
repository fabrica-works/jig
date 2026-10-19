//! Results read after the fleet's fence (protocol/channel.md, section 6.1;
//! domain/hosts.md, section 8). The routing maps these into domain values.
use alloc::boxed::Box;
use skein_lib::{List, Reader};
use smith_charter::{Form, v3};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Field {
    pub name: Box<[u8]>,
    pub value: Box<[u8]>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RunResult {
    Report { words: Box<[u8]> },
    Verdict { label: Box<[u8]>, fields: Box<[Field]>, words: Box<[u8]> },
    Failure { reason: Box<[u8]> },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unreadable {
    Malformed,
    Form,
}

pub fn result(bytes: &[u8]) -> Result<RunResult, Unreadable> {
    let mut reader = Reader::new(bytes);
    let Ok(value) = smith_charter::RunResult::decode(&v3::CEILINGS, &mut reader) else {
        return Err(Unreadable::Malformed);
    };
    if !reader.is_empty() {
        return Err(Unreadable::Malformed);
    }
    match value.form() {
        Form::Change => Err(Unreadable::Form),
        Form::Report => {
            if value.label().is_some() || !value.items().is_empty() {
                return Err(Unreadable::Form);
            }
            Ok(RunResult::Report { words: Box::from(value.text()) })
        }
        Form::Failure => {
            if value.label().is_some() || !value.items().is_empty() {
                return Err(Unreadable::Form);
            }
            Ok(RunResult::Failure { reason: Box::from(value.text()) })
        }
        Form::Verdict => {
            if !value.items().is_empty() {
                return Err(Unreadable::Form);
            }
            let Some(label) = value.label() else {
                return Err(Unreadable::Form);
            };
            let mut fields = List::with_capacity(value.fields().len());
            for field in value.fields() {
                fields
                    .push(Field { name: Box::from(field.name()), value: Box::from(field.text()) })
                    .expect("capacity for every decoded field");
            }
            Ok(RunResult::Verdict { label: label.clone(), fields: fields.into_boxed(), words: Box::from(value.text()) })
        }
    }
}
