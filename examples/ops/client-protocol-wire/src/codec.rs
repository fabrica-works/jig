//! Whole, versioned records at both ends of the family wire (protocol/client.md, 11).
use crate::v1 as w;
use alloc::boxed::Box;
use skein_lib::{Reader, Writer};

/// A record with the wrong version, malformed fields, or trailing bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Undecodable;

pub mod infrastructure {
    use super::{Box, Reader, Undecodable, Writer, w};

    /// Encode an admitted environment request.
    #[must_use]
    pub fn ask(value: &w::InfrastructureAskRecord) -> Box<[u8]> {
        let mut writer = Writer::new(usize::try_from(value.measure()).expect("bounded request"));
        value.encode(&mut writer).expect("measured request");
        writer.finish()
    }

    /// Decode a request before the engine admits it.
    pub fn ask_value(bytes: &[u8]) -> Result<w::InfrastructureAskRecord, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::InfrastructureAskRecord::decode(&w::CEILINGS, &mut reader) else { return Err(Undecodable) };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }

    /// Encode the connector's durable answer.
    #[must_use]
    pub fn answer_record(value: &w::InfrastructureAnswerRecord) -> Box<[u8]> {
        let mut writer = Writer::new(usize::try_from(value.measure()).expect("bounded answer"));
        value.encode(&mut writer).expect("measured answer");
        writer.finish()
    }

    /// Decode the connector's answer at the client.
    pub fn answer(bytes: &[u8]) -> Result<w::InfrastructureAnswerRecord, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::InfrastructureAnswerRecord::decode(&w::CEILINGS, &mut reader) else {
            return Err(Undecodable);
        };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }

    /// Encode a scoped subject for a watch or read.
    #[must_use]
    pub fn subject(value: &w::InfrastructureSubjectRecord) -> Box<[u8]> {
        let mut writer = Writer::new(usize::try_from(value.measure()).expect("bounded subject"));
        value.encode(&mut writer).expect("measured subject");
        writer.finish()
    }

    /// Decode the subject at the engine.
    pub fn subject_value(bytes: &[u8]) -> Result<w::InfrastructureSubjectRecord, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::InfrastructureSubjectRecord::decode(&w::CEILINGS, &mut reader) else {
            return Err(Undecodable);
        };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }

    /// Encode the owner's whole document.
    #[must_use]
    pub fn document_record(value: &w::InfrastructureDocumentRecord) -> Box<[u8]> {
        let mut writer = Writer::new(usize::try_from(value.measure()).expect("bounded document"));
        value.encode(&mut writer).expect("measured document");
        writer.finish()
    }

    /// Decode the owner's whole document at the client.
    pub fn document(bytes: &[u8]) -> Result<w::InfrastructureDocumentRecord, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::InfrastructureDocumentRecord::decode(&w::CEILINGS, &mut reader) else {
            return Err(Undecodable);
        };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }
    /// Encode the owner's read answer, including a refusal.
    #[must_use]
    pub fn read_answer(value: &w::InfrastructureReadAnswer) -> Box<[u8]> {
        let mut writer = Writer::new(usize::try_from(value.measure()).expect("bounded read answer"));
        value.encode(&mut writer).expect("measured read answer");
        writer.finish()
    }
    /// Decode a whole owner read answer.
    pub fn read_answer_value(bytes: &[u8]) -> Result<w::InfrastructureReadAnswer, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::InfrastructureReadAnswer::decode(&w::CEILINGS, &mut reader) else {
            return Err(Undecodable);
        };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }
}

pub mod observability {
    use super::{Box, Reader, Undecodable, Writer, w};

    /// Encode a scoped incident, alert or panel subject.
    #[must_use]
    pub fn subject(value: &w::ObservabilitySubjectRecord) -> Box<[u8]> {
        let mut writer = Writer::new(usize::try_from(value.measure()).expect("bounded subject"));
        value.encode(&mut writer).expect("measured subject");
        writer.finish()
    }

    /// Decode the subject at the engine.
    pub fn subject_value(bytes: &[u8]) -> Result<w::ObservabilitySubjectRecord, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::ObservabilitySubjectRecord::decode(&w::CEILINGS, &mut reader) else {
            return Err(Undecodable);
        };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }

    /// Encode the owner's whole incident, alert or panel document.
    #[must_use]
    pub fn document_record(value: &w::ObservabilityDocumentRecord) -> Box<[u8]> {
        let mut writer = Writer::new(usize::try_from(value.measure()).expect("bounded document"));
        value.encode(&mut writer).expect("measured document");
        writer.finish()
    }

    /// Decode the owner's whole document at the client.
    pub fn document(bytes: &[u8]) -> Result<w::ObservabilityDocumentRecord, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::ObservabilityDocumentRecord::decode(&w::CEILINGS, &mut reader) else {
            return Err(Undecodable);
        };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }
    /// Encode the owner's read answer, including a refusal.
    #[must_use]
    pub fn read_answer(value: &w::ObservabilityReadAnswer) -> Box<[u8]> {
        let mut writer = Writer::new(usize::try_from(value.measure()).expect("bounded read answer"));
        value.encode(&mut writer).expect("measured read answer");
        writer.finish()
    }
    /// Decode a whole owner read answer.
    pub fn read_answer_value(bytes: &[u8]) -> Result<w::ObservabilityReadAnswer, Undecodable> {
        let mut reader = Reader::new(bytes);
        let Ok(record) = w::ObservabilityReadAnswer::decode(&w::CEILINGS, &mut reader) else {
            return Err(Undecodable);
        };
        if reader.remaining() != 0 {
            return Err(Undecodable);
        }
        Ok(record)
    }
}
