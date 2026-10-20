//! The accounts family's rows (protocol/store.md, 4.4). Its one row so far is
//! the installation's identifier; the accounts' own rows join this version.
use crate::accounts_v1 as wire;
use alloc::boxed::Box;
use skein_lib::{Reader, Writer};

/// The installation's identifier, drawn at the first start (configuration.md, 8).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Installation {
    pub id: [u8; 16],
}

impl Installation {
    /// Encode this row with the accounts family's version.
    #[must_use]
    pub fn encode(&self) -> Box<[u8]> {
        let row = wire::Installation::new(&wire::CEILINGS, wire::InstallationParts { id: self.id });
        let mut writer = Writer::new(usize::try_from(row.measure()).expect("encoded row fits usize"));
        row.encode(&mut writer).expect("measured room");
        writer.finish()
    }

    /// Decode one row, refusing another version or trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let wire::InstallationParts { id } =
            wire::Installation::decode(&wire::CEILINGS, &mut Reader::new(bytes))?.into_parts();
        Ok(Self { id })
    }
}
