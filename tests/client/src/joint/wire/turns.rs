//! Whole committed turn decoding through the actual codec (client.md, 10).
//! Provisional completion identities and offsets never become committed names.
use jig_client_domain::contract::Turn;
use jig_protocol_transcript::{Limits, Undecodable};

/// # Errors
/// Returns the actual codec error for unsupported versions, malformed bytes or a record above its admitted bounds.
pub fn turn(bytes: &[u8], limits: &Limits) -> Result<Turn, Undecodable> {
    jig_protocol_transcript::decode(limits, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_client_domain::contract::{Block, TurnLimits};

    fn record() -> &'static [u8] {
        include_bytes!("../../../../../crates/jig-protocol-transcript/tests/data/record_turn_full.bin")
    }

    fn limits() -> Limits {
        Limits {
            smith: smith_transcript::v3::CEILINGS,
            bytes: 142,
            turn: TurnLimits { messages: 1, blocks: 1, text: 3 },
        }
    }

    #[test]
    fn committed_turn_bytes_use_the_whole_actual_codec_and_preserve_usage_without_a_provisional_retry_or_read_position()
    {
        let bytes = record();
        let result = turn(bytes, &limits()).expect("actual whole committed record");
        assert_eq!(result.used.input, u64::MAX);
        assert_eq!(result.used.output, u64::MAX);
        assert_eq!(result.used.cached, u64::MAX);
        assert_eq!(result.used.cache_written, u64::MAX);
        assert_eq!(result.spent, u64::MAX);
        assert_eq!(result.read_through, None);
        assert!(!result.cut);
        let Block::Text(words) = result.messages.get(0).expect("actual message").blocks.get(0).expect("actual text")
        else {
            panic!("the codec fixture supplies one text block")
        };
        assert_eq!(words.bytes(), b"abc");
        let short = Limits { turn: TurnLimits { text: 1, ..limits().turn }, ..limits() };
        let short = turn(bytes, &short).expect("receiving display loss remains explicit");
        assert!(short.cut);
        assert_eq!(short.used, result.used);
        assert_eq!(short.spent, result.spent);
        assert_eq!(short.read_through, None);
    }

    #[test]
    fn committed_codec_admission_rejects_whole_oversize_trailing_bytes_and_version_without_mutating_the_record() {
        let original = record().to_vec();
        assert_eq!(turn(&original, &Limits { bytes: 141, ..limits() }), Err(Undecodable::TooLarge));
        let mut trailing = original.clone();
        trailing.push(0);
        assert_eq!(turn(&trailing, &Limits { bytes: 143, ..limits() }), Err(Undecodable::Malformed));
        assert_eq!(turn(&[255, 255], &limits()), Err(Undecodable::Version { found: u16::MAX }));
        assert_eq!(original, record());
    }
}
