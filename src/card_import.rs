//! Bounded host imports. HPP titles are presentation, never physical identity.
use std::{fs::File, io::Read, path::PathBuf};
use hp67emu::machines::hp67::{
    CardInsertionEnd, Hp67MagneticCard, TeenixHppImport,
};

const MAX_HPP_BYTES: usize = 1024 * 1024;

pub struct ImportedCard {
    pub card: Hp67MagneticCard,
    pub name: Option<String>,
    pub end: CardInsertionEnd,
    pub warnings: Vec<String>,
}

fn extension(path: &std::path::Path) -> String {
    path.extension().and_then(|x| x.to_str()).unwrap_or_default().to_ascii_lowercase()
}

fn read_bounded(reader: impl Read, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader.take(limit as u64 + 1).read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read card: {error}"))?;
    if bytes.len() > limit {
        return Err(format!("card file exceeds {limit} bytes"));
    }
    Ok(bytes)
}

fn decode(kind: &str, bytes: &[u8]) -> Result<ImportedCard, String> {
    let mut warnings = Vec::new();
    let (card, name, end) = match kind {
        "hpp" => {
            let imported = TeenixHppImport::from_bytes(bytes)
                .map_err(|error| format!("invalid Teenix .hpp: {error:?}"))?;
            if !imported.length_matches {
                warnings.push("HPP declared length differs from decoded content".to_owned());
            }
            let end = CardInsertionEnd::for_track(imported.card_track);
            let card = Hp67MagneticCard::default().with_track(imported.card_track, imported.track);
            (card, Some(imported.card_name), end)
        }
        "hp67card" => (
            Hp67MagneticCard::from_hp67card_bytes(bytes).map_err(|e| format!("invalid .hp67card: {e:?}"))?,
            None, CardInsertionEnd::End1,
        ),
        "hp67raw" => (
            Hp67MagneticCard::from_hp67raw_bytes(bytes).map_err(|e| format!("invalid .hp67raw: {e:?}"))?,
            None, CardInsertionEnd::End1,
        ),
        _ => return Err("expected .hpp, .hp67raw or .hp67card".to_owned()),
    };
    Ok(ImportedCard { card, name, end, warnings })
}

fn pair(mut first: ImportedCard, second: ImportedCard) -> Result<ImportedCard, String> {
    if first.end == second.end {
        return Err("Both HPP files identify the same track. Convert with explicit --track1/--track2 placement before importing.".to_owned());
    }
    first.card.set_track(second.end.track(), second.card.track(second.end.track()).clone());
    first.warnings.extend(second.warnings);
    if first.name != second.name {
        first.name = None;
    }
    first.end = CardInsertionEnd::End1;
    Ok(first)
}

/// One file creates a new physical card. Two HPP files in the same operation
/// explicitly request pairing; neither names nor previous UI media are consulted.
pub fn load(paths: &[PathBuf]) -> Result<ImportedCard, String> {
    if paths.is_empty() || paths.len() > 2 {
        return Err("Drop one card file, or exactly two HPP files together to pair tracks.".to_owned());
    }
    if paths.len() == 2 && paths.iter().any(|p| extension(p) != "hpp") {
        return Err("Only two HPP files can be paired. Import native/raw cards separately.".to_owned());
    }
    let mut imports = Vec::new();
    for path in paths {
        let kind = extension(path);
        let limit = match kind.as_str() {
            "hpp" => MAX_HPP_BYTES,
            "hp67card" => 250,
            "hp67raw" => 238,
            _ => return Err(format!("{}: expected .hpp, .hp67raw or .hp67card", path.display())),
        };
        let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let bytes = read_bounded(file, limit).map_err(|e| format!("{}: {e}", path.display()))?;
        imports.push(decode(&kind, &bytes).map_err(|e| format!("{}: {e}", path.display()))?);
    }
    let first = imports.remove(0);
    match imports.pop() {
        Some(second) => pair(first, second),
        None => Ok(first),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hp67emu::machines::hp67::Hp67CardTrack;

    fn hpp(header: u32, name: &str) -> Vec<u8> {
        let mut words = [0x1234567_u32; 34];
        words[0] = header;
        let mut data = "0".repeat(21);
        for word in words.into_iter().chain(std::iter::once(words[33])) {
            for shift in 0..7 {
                data.push(char::from_digit((word >> (shift * 4)) & 15, 16).unwrap());
            }
        }
        let body = format!("67\ncard.bmp\n{name}\n{data}\n");
        format!("NeWe\n{}\n{body}", body.len()).bytes().map(|b| b ^ 0x55).collect()
    }

    #[test]
    fn core_audit_hpp_same_title_does_not_merge_separate_imports() {
        let first = decode("hpp", &hpp(0x03000222, "Same title")).unwrap();
        let second = decode("hpp", &hpp(0x04000222, "Same title")).unwrap();
        assert!(first.card.track(Hp67CardTrack::Track1).is_recorded());
        assert!(!second.card.track(Hp67CardTrack::Track1).is_recorded());
        assert!(second.card.track(Hp67CardTrack::Track2).is_recorded());
    }

    #[test]
    fn core_audit_hpp_explicit_pair_preserves_both_tracks_in_either_order() {
        for reverse in [false, true] {
            let a = decode("hpp", &hpp(0x03000222, "A")).unwrap();
            let b = decode("hpp", &hpp(0x04000222, "B")).unwrap();
            let expected_a = a.card.track(Hp67CardTrack::Track1).clone();
            let expected_b = b.card.track(Hp67CardTrack::Track2).clone();
            let result = if reverse { pair(b, a) } else { pair(a, b) }.unwrap();
            assert_eq!(result.card.track(Hp67CardTrack::Track1), &expected_a);
            assert_eq!(result.card.track(Hp67CardTrack::Track2), &expected_b);
            assert_eq!(result.end, CardInsertionEnd::End1);
            assert!(result.name.is_none());
        }
        assert!(pair(
            decode("hpp", &hpp(0x03000222, "A")).unwrap(),
            decode("hpp", &hpp(0x03000222, "A")).unwrap(),
        ).is_err());
    }

    #[test]
    fn core_audit_card_reader_limits_actual_bytes_and_rejects_ambiguous_batches() {
        for limit in [238, 250, MAX_HPP_BYTES] {
            assert_eq!(read_bounded(std::io::repeat(0).take(limit as u64), limit).unwrap().len(), limit);
            assert!(read_bounded(std::io::repeat(0), limit).is_err());
        }
        assert!(load(&[]).is_err());
        assert!(load(&[PathBuf::from("a.hp67card"), PathBuf::from("b.hpp")]).is_err());
        assert!(load(&[PathBuf::from("a.hpp"), PathBuf::from("b.hpp"), PathBuf::from("c.hpp")]).is_err());
        assert!(decode("hp67card", &[0; 249]).is_err());
        assert!(decode("hp67raw", &[0; 237]).is_err());
    }
    #[test]
    fn core_audit_card_import_validates_complete_disk_batch_before_returning_media() {
        let root = std::env::temp_dir().join(format!("hp67-import-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let a = root.join("first.hpp");
        let b = root.join("second.hpp");
        std::fs::write(&a, hpp(0x03000222, "Same title")).unwrap();
        std::fs::write(&b, hpp(0x04000222, "Same title")).unwrap();
        let pair = load(&[a.clone(), b.clone()]).unwrap();
        assert!(pair.card.track(Hp67CardTrack::Track1).is_recorded());
        assert!(pair.card.track(Hp67CardTrack::Track2).is_recorded());
        std::fs::write(&b, b"malformed second file").unwrap();
        assert!(load(&[a.clone(), b.clone()]).is_err());
        assert!(load(&[a]).is_ok());
        let native = root.join("card.hp67card");
        let bytes = pair.card.to_hp67card_bytes();
        std::fs::write(&native, bytes).unwrap();
        assert_eq!(load(&[native.clone()]).unwrap().card, pair.card);
        let mut oversized = bytes.to_vec();
        oversized.push(0);
        std::fs::write(&native, oversized).unwrap();
        assert!(load(&[native]).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

}
