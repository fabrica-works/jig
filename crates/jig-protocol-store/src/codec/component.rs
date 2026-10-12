//! The component's own durable rows (protocol/store.md, sections 5.3 and 8.1).
use crate::component_v1 as wire;
use alloc::boxed::Box;
use skein_lib::{Reader, Writer};

/// The component's format row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Format {
    pub component: u16,
    pub core: u16,
    pub tasks: u16,
    pub people: u16,
    pub notes: u16,
    pub application: u32,
    pub deployment: [u8; 16],
}

impl Format {
    /// Encode this row with the component family's version.
    pub fn encode(&self) -> Result<Box<[u8]>, wire::Problem> {
        let row = wire::Format::new(
            &wire::CEILINGS,
            wire::FormatParts {
                component: self.component,
                core: self.core,
                tasks: self.tasks,
                people: self.people,
                notes: self.notes,
                application: self.application,
                deployment: self.deployment,
            },
        );
        let mut writer = Writer::new(usize::try_from(row.measure()).expect("encoded row fits usize"));
        row.encode(&mut writer).expect("measured room");
        Ok(writer.finish())
    }

    /// Decode one row, refusing another version or trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let wire::FormatParts { component, core, tasks, people, notes, application, deployment } =
            wire::Format::decode(&wire::CEILINGS, &mut Reader::new(bytes))?.into_parts();
        Ok(Self { component, core, tasks, people, notes, application, deployment })
    }
}

/// The component's transcript row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Transcript {
    pub length: u64,
    pub window: u64,
}

impl Transcript {
    /// Encode this row with the component family's version.
    pub fn encode(&self) -> Result<Box<[u8]>, wire::Problem> {
        let row =
            wire::Transcript::new(&wire::CEILINGS, wire::TranscriptParts { length: self.length, window: self.window });
        let mut writer = Writer::new(usize::try_from(row.measure()).expect("encoded row fits usize"));
        row.encode(&mut writer).expect("measured room");
        Ok(writer.finish())
    }

    /// Decode one row, refusing another version or trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let wire::TranscriptParts { length, window } =
            wire::Transcript::decode(&wire::CEILINGS, &mut Reader::new(bytes))?.into_parts();
        Ok(Self { length, window })
    }
}

/// The component's attempt row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Attempt {
    pub first: u64,
}

impl Attempt {
    /// Encode this row with the component family's version.
    pub fn encode(&self) -> Result<Box<[u8]>, wire::Problem> {
        let row = wire::Attempt::new(&wire::CEILINGS, wire::AttemptParts { first: self.first });
        let mut writer = Writer::new(usize::try_from(row.measure()).expect("encoded row fits usize"));
        row.encode(&mut writer).expect("measured room");
        Ok(writer.finish())
    }

    /// Decode one row, refusing another version or trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let wire::AttemptParts { first } =
            wire::Attempt::decode(&wire::CEILINGS, &mut Reader::new(bytes))?.into_parts();
        Ok(Self { first })
    }
}
