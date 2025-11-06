use crate::extractors::common::{Chroot, ExtractionResult, Extractor, ExtractorType};
use crate::structures::cramfs::parse_cramfs_header;

const OUTPUT_BASENAME: &str = "filesystem";

/// Provides an internal extractor for CramFS images.
pub fn cramfs_extractor() -> Extractor {
    Extractor {
        utility: ExtractorType::Internal(extract_cramfs_image),
        extension: "cramfs".to_string(),
        ..Default::default()
    }
}

fn extract_cramfs_image(
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

    let remaining = &file_data[offset..];

    if let Ok(header) = parse_cramfs_header(remaining) {
        let available = file_data.len() - offset;
        let reported_size = header.size;
        let carve_size = reported_size.min(available);

        result.size = Some(carve_size);

        match output_directory {
            None => {
                result.success = true;
            }
            Some(out_dir) => {
                let chroot = Chroot::new(Some(out_dir));
                let output_name = format!("{OUTPUT_BASENAME}_{offset:08x}.cramfs");
                result.success = chroot.carve_file(&output_name, file_data, offset, carve_size);
            }
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
    fn dry_run_succeeds_for_valid_image() {
        let data = std::fs::read("tests/inputs/cramfs.bin").expect("test data");
        let offset = 0;

        let result = extract_cramfs_image(&data, offset, None);
        assert!(result.success);
        assert!(result.size.is_some());
    }

    #[test]
    fn extractor_is_registered() {
        let binwalker = Binwalk::new();
        let extractor = binwalker
            .extractor_lookup_table
            .get("cramfs")
            .expect("cramfs extractor entry");

        match extractor {
            Some(definition) => match &definition.utility {
                ExtractorType::Internal(_) => {}
                other => panic!("expected internal extractor, found {other:?}"),
            },
            None => panic!("missing cramfs extractor"),
        }
    }
}
