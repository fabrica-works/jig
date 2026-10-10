//! Text loss is supplied by the document owner (client/domain/contract.md, 5.2–5.3).
use super::Text;

/// One document text value with its independent loss fact.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DocumentText {
    pub bytes: Text,
    pub cut: bool,
}
impl DocumentText {
    /// A caller supplies a complete value, with no omitted bytes.
    #[must_use]
    pub fn complete(bytes: Text) -> Self {
        Self { bytes, cut: false }
    }
    /// The supplied bytes, including a partial value when marked cut.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }
    /// A complete request value is available only without document loss.
    #[must_use]
    pub fn whole(&self) -> Option<Text> {
        if self.cut { None } else { Some(self.bytes.clone()) }
    }
}
