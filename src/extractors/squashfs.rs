use crate::extractors::common::{Chroot, ExtractionResult, Extractor, ExtractorType};
use crate::structures::squashfs::parse_squashfs_header;

const OUTPUT_BASENAME: &str = "squashfs";

fn internal_extractor() -> Extractor {
    Extractor {
        utility: ExtractorType::Internal(extract_squashfs_image),
        extension: "sqsh".to_string(),
        ..Default::default()
    }
}

fn extract_squashfs_image(
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

    if let Ok(header) = parse_squashfs_header(&file_data[offset..]) {
        let available = file_data.len() - offset;
        let image_size = header.image_size.min(available);
        result.size = Some(image_size);

        match output_directory {
            None => {
                result.success = true;
            }
            Some(out_dir) => {
                let chroot = Chroot::new(Some(out_dir));
                let output_name = format!("{OUTPUT_BASENAME}_{offset:08x}.sqsh");
                result.success = chroot.carve_file(&output_name, file_data, offset, image_size);
            }
        }
    }

    result
}

/// Returns an internal extractor for SquashFS images.
pub fn squashfs_extractor() -> Extractor {
    internal_extractor()
}

/// Returns an internal extractor for little-endian SquashFS images.
pub fn squashfs_le_extractor() -> Extractor {
    internal_extractor()
}

/// Returns an internal extractor for big-endian SquashFS images.
pub fn squashfs_be_extractor() -> Extractor {
    internal_extractor()
}

/// Returns an internal extractor for SquashFS v4 big-endian images.
pub fn squashfs_v4_be_extractor() -> Extractor {
    internal_extractor()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binwalk::Binwalk;
    use crate::extractors::common::ExtractorType;

    #[test]
    fn dry_run_uses_reported_image_size() {
        let data = std::fs::read("tests/inputs/squashfs.bin").expect("test data");
        let result = extract_squashfs_image(&data, 0, None);
        assert!(result.success);
        assert!(result.size.unwrap() <= data.len());
    }

    #[test]
    fn extractor_is_internal() {
        let binwalker = Binwalk::new();
        let extractor = binwalker
            .extractor_lookup_table
            .get("squashfs")
            .expect("squashfs extractor entry");

        match extractor {
            Some(definition) => match &definition.utility {
                ExtractorType::Internal(_) => {}
                other => panic!("expected internal extractor, found {other:?}"),
            },
            None => panic!("missing squashfs extractor"),
        }
    }
}
