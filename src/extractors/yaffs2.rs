use crate::extractors::common::{Chroot, ExtractionResult, Extractor, ExtractorType};

const OUTPUT_BASENAME: &str = "yaffs2";

fn internal_extractor() -> Extractor {
    Extractor {
        utility: ExtractorType::Internal(extract_yaffs2_image),
        extension: "img".to_string(),
        ..Default::default()
    }
}

fn extract_yaffs2_image(
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

    let available = file_data.len() - offset;
    result.size = Some(available);

    match output_directory {
        None => {
            result.success = true;
        }
        Some(out_dir) => {
            let chroot = Chroot::new(Some(out_dir));
            let output_name = format!("{OUTPUT_BASENAME}_{offset:08x}.img");
            result.success = chroot.carve_file(&output_name, file_data, offset, available);
        }
    }

    result
}

/// Returns an internal extractor for YAFFS2 images.
pub fn yaffs2_extractor() -> Extractor {
    internal_extractor()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binwalk::Binwalk;
    use crate::extractors::common::ExtractorType;

    #[test]
    fn dry_run_reports_available_size() {
        let data = std::fs::read("tests/inputs/yaffs2.bin").expect("test data");
        let result = extract_yaffs2_image(&data, 0, None);
        assert!(result.success);
        assert_eq!(result.size, Some(data.len()));
    }

    #[test]
    fn extractor_is_internal() {
        let binwalker = Binwalk::new();
        let extractor = binwalker
            .extractor_lookup_table
            .get("yaffs")
            .expect("yaffs extractor entry");

        match extractor {
            Some(definition) => match &definition.utility {
                ExtractorType::Internal(_) => {}
                other => panic!("expected internal extractor, found {other:?}"),
            },
            None => panic!("missing yaffs extractor"),
        }
    }
}
