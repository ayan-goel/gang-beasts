use brawler_engine::{ENGINE_HASH, MapDefinition};
use std::sync::OnceLock;

pub(super) fn compatible(hashes: &[u8; 128]) -> bool {
    static MAP_HASH: OnceLock<String> = OnceLock::new();
    &hashes[..64] == ENGINE_HASH.as_bytes()
        && &hashes[64..] == MAP_HASH.get_or_init(MapDefinition::skyline_hash).as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use brawler_protocol::lobby::Command;

    #[test]
    fn both_bundled_digests_and_the_complete_record_are_required() {
        let mut record = vec![20, 1];
        record.extend(ENGINE_HASH.as_bytes());
        record.extend(MapDefinition::skyline_hash().bytes());
        let matches = |r: &[u8]| matches!(Command::decode(r), Some(Command::MapLoaded(hashes)) if compatible(hashes));
        assert!(matches(&record));
        for offset in [0, 1, 2, 65, 66, 129] {
            let mut other = record.clone();
            other[offset] ^= 1;
            assert!(!matches(&other));
        }
        assert!(!matches(&record[..129]));
        record.push(0);
        assert!(!matches(&record));
    }
}
