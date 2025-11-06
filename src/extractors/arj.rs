use crate::extractors::common::{Chroot, ExtractionResult, Extractor, ExtractorType};
use crate::structures::arj::parse_arj_header;

const DEFAULT_OUTPUT_NAME: &str = "arj_archive";

/// Provides an internal extractor for ARJ archives.
pub fn arj_extractor() -> Extractor {
    Extractor {
        utility: ExtractorType::Internal(extract_arj_archive),
        extension: "arj".to_string(),
        ..Default::default()
    }
}

fn build_output_name(file_data: &[u8], offset: usize) -> String {
    if let Ok(header) = parse_arj_header(&file_data[offset..]) {
        if !header.original_name.is_empty() {
            let sanitized = header.original_name.replace(['/', '\\'], "_");
            return format!("{sanitized}_{offset:08x}.arj");
        }
    }

    format!("{DEFAULT_OUTPUT_NAME}_{offset:08x}.arj")
}

fn extract_arj_archive(
    file_data: &[u8],
    offset: usize,
    output_directory: Option<&str>,
) -> ExtractionResult {
    let mut result = ExtractionResult {
        ..Default::default()
    };

    if offset >= file_data.len() {
        return result;
    }

    let remaining = file_data.len() - offset;
    result.size = Some(remaining);

    match output_directory {
        None => {
            // Dry-run validation.
            result.success = true;
        }
        Some(out_dir) => {
            let chroot = Chroot::new(Some(out_dir));
            let output_name = build_output_name(file_data, offset);

            result.success = chroot.carve_file(&output_name, file_data, offset, remaining);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binwalk::Binwalk;
    use crate::extractors::common::ExtractorType;

    #[test]
    fn carve_entire_archive_when_output_requested() {
        let data = std::fs::read("tests/inputs/arj.bin").expect("test data");
        let offset = 0x0d;

        let output_dir = std::env::temp_dir().join("binwalk_arj_extractor_test");
        let _ = std::fs::remove_dir_all(&output_dir);

        let output_str = output_dir.to_str().unwrap();

        let result = extract_arj_archive(&data, offset, Some(output_str));

        assert!(result.success);

        let extracted_files: Vec<_> = std::fs::read_dir(&output_dir).unwrap().collect();
        assert!(!extracted_files.is_empty());

        for entry in extracted_files {
            let entry = entry.expect("dir entry");
            let meta = entry.metadata().unwrap();
            assert!(meta.len() > 0);
        }

        let _ = std::fs::remove_dir_all(&output_dir);
    }

    #[test]
    fn arj_signature_uses_internal_extractor() {
        let binwalker = Binwalk::new();
        let extractor = binwalker
            .extractor_lookup_table
            .get("arj")
            .expect("arj extractor entry");

        match extractor {
            Some(definition) => match &definition.utility {
                ExtractorType::Internal(_) => {}
                other => panic!("expected internal extractor, found {other:?}"),
            },
            None => panic!("missing arj extractor"),
        }
    }
}
