use std::collections::HashSet;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PromptBatch {
    requests: Vec<PromptRequest>,
}

impl PromptBatch {
    pub(crate) fn new(
        prompt: &str,
        out: &Path,
        variants: &[String],
        variant_separator: Option<&str>,
        n: Option<u64>,
    ) -> Result<Self> {
        let sample_count = n.unwrap_or(1);
        if sample_count == 0 {
            bail!("--n must be greater than 0");
        }

        for variant in variants {
            if variant.is_empty() {
                bail!("--variant cannot be empty");
            }
        }

        let separator = decode_separator(variant_separator)?;
        let requests = if variants.is_empty() {
            vec![PromptRequest {
                prompt: prompt.to_string(),
                n,
                outputs: sample_outputs(out, sample_count)?,
            }]
        } else {
            variant_requests(prompt, out, variants, &separator, sample_count, n)?
        };

        assert_unique_outputs(&requests)?;
        Ok(Self { requests })
    }

    pub(crate) fn requests(&self) -> &[PromptRequest] {
        &self.requests
    }

    pub(crate) fn single_output_jobs(&self) -> Vec<SingleOutputJob> {
        self.requests
            .iter()
            .flat_map(|request| {
                request.outputs.iter().cloned().map(|out| SingleOutputJob {
                    prompt: request.prompt.clone(),
                    out,
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PromptRequest {
    pub(crate) prompt: String,
    pub(crate) n: Option<u64>,
    pub(crate) outputs: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SingleOutputJob {
    pub(crate) prompt: String,
    pub(crate) out: PathBuf,
}

fn variant_requests(
    prompt: &str,
    out: &Path,
    variants: &[String],
    separator: &str,
    sample_count: u64,
    n: Option<u64>,
) -> Result<Vec<PromptRequest>> {
    let variant_width = decimal_width(variants.len() as u64, 3);
    let sample_width = decimal_width(sample_count, 2);

    variants
        .iter()
        .enumerate()
        .map(|(variant_index, variant)| {
            let variant_number = variant_index as u64 + 1;
            let prompt = format!("{prompt}{separator}{variant}");
            let outputs = if sample_count == 1 {
                vec![suffixed_output_path(
                    out,
                    &format!("{variant_number:0variant_width$}"),
                )?]
            } else {
                (1..=sample_count)
                    .map(|sample_number| {
                        suffixed_output_path(
                            out,
                            &format!(
                                "{variant_number:0variant_width$}-{sample_number:0sample_width$}"
                            ),
                        )
                    })
                    .collect::<Result<Vec<_>>>()?
            };

            Ok(PromptRequest { prompt, n, outputs })
        })
        .collect()
}

fn sample_outputs(out: &Path, sample_count: u64) -> Result<Vec<PathBuf>> {
    if sample_count == 1 {
        return Ok(vec![out.to_path_buf()]);
    }

    let sample_width = decimal_width(sample_count, 3);
    (1..=sample_count)
        .map(|sample_number| suffixed_output_path(out, &format!("{sample_number:0sample_width$}")))
        .collect()
}

fn suffixed_output_path(out: &Path, suffix: &str) -> Result<PathBuf> {
    let stem = out.file_stem().with_context(|| {
        format!(
            "failed to derive batch output path from `{}`",
            out.display()
        )
    })?;
    let mut file_name = OsString::from(stem);
    file_name.push(format!("-{suffix}"));
    if let Some(extension) = out.extension() {
        file_name.push(".");
        file_name.push(extension);
    }

    Ok(out.with_file_name(file_name))
}

fn decimal_width(value: u64, minimum: usize) -> usize {
    value.to_string().len().max(minimum)
}

fn decode_separator(raw: Option<&str>) -> Result<String> {
    let Some(raw) = raw else {
        return Ok("\n".to_string());
    };

    let mut decoded = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            decoded.push(ch);
            continue;
        }

        let Some(escaped) = chars.next() else {
            decoded.push('\\');
            break;
        };
        match escaped {
            'n' => decoded.push('\n'),
            'r' => decoded.push('\r'),
            't' => decoded.push('\t'),
            '\\' => decoded.push('\\'),
            other => {
                decoded.push('\\');
                decoded.push(other);
            }
        }
    }
    Ok(decoded)
}

fn assert_unique_outputs(requests: &[PromptRequest]) -> Result<()> {
    let mut seen = HashSet::new();
    for output in requests
        .iter()
        .flat_map(|request| request.outputs.iter().cloned())
    {
        if !seen.insert(output.clone()) {
            bail!("batch output path collision: `{}`", output.display());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn variants_append_to_base_prompt_with_default_newline_separator() {
        let variants = vec![
            "matte black, hard side light".to_string(),
            "pale blue".to_string(),
        ];

        let batch = PromptBatch::new(
            "studio product photo of a ceramic teapot",
            Path::new("teapot.png"),
            &variants,
            None,
            None,
        )
        .unwrap();

        assert_eq!(
            batch.requests(),
            [
                PromptRequest {
                    prompt:
                        "studio product photo of a ceramic teapot\nmatte black, hard side light"
                            .to_string(),
                    n: None,
                    outputs: vec![PathBuf::from("teapot-001.png")],
                },
                PromptRequest {
                    prompt: "studio product photo of a ceramic teapot\npale blue".to_string(),
                    n: None,
                    outputs: vec![PathBuf::from("teapot-002.png")],
                },
            ]
        );
    }

    #[test]
    fn variant_separator_is_configurable_and_decodes_shell_friendly_escapes() {
        let variants = vec!["macro lens".to_string()];

        let batch = PromptBatch::new(
            "watch face",
            Path::new("watch.png"),
            &variants,
            Some("\\n\\n"),
            None,
        )
        .unwrap();

        assert_eq!(batch.requests()[0].prompt, "watch face\n\nmacro lens");
    }

    #[test]
    fn variant_separator_preserves_arbitrary_backslash_text() {
        let variants = vec!["macro lens".to_string()];

        let batch = PromptBatch::new(
            "watch face",
            Path::new("watch.png"),
            &variants,
            Some("\\q \\"),
            None,
        )
        .unwrap();

        assert_eq!(batch.requests()[0].prompt, "watch face\\q \\macro lens");
    }

    #[test]
    fn n_without_variants_allocates_suffixed_outputs_for_one_prompt() {
        let batch =
            PromptBatch::new("portrait", Path::new("portrait.png"), &[], None, Some(3)).unwrap();

        assert_eq!(
            batch.requests(),
            [PromptRequest {
                prompt: "portrait".to_string(),
                n: Some(3),
                outputs: vec![
                    PathBuf::from("portrait-001.png"),
                    PathBuf::from("portrait-002.png"),
                    PathBuf::from("portrait-003.png"),
                ],
            }]
        );
    }

    #[test]
    fn variants_and_n_allocate_variant_and_sample_suffixes() {
        let variants = vec!["red".to_string(), "blue".to_string()];

        let batch = PromptBatch::new(
            "product photo",
            Path::new("item.png"),
            &variants,
            None,
            Some(2),
        )
        .unwrap();

        assert_eq!(
            batch.requests(),
            [
                PromptRequest {
                    prompt: "product photo\nred".to_string(),
                    n: Some(2),
                    outputs: vec![
                        PathBuf::from("item-001-01.png"),
                        PathBuf::from("item-001-02.png")
                    ],
                },
                PromptRequest {
                    prompt: "product photo\nblue".to_string(),
                    n: Some(2),
                    outputs: vec![
                        PathBuf::from("item-002-01.png"),
                        PathBuf::from("item-002-02.png")
                    ],
                },
            ]
        );
    }

    #[test]
    fn single_output_keeps_exact_path() {
        let batch = PromptBatch::new("one image", Path::new("exact.png"), &[], None, None).unwrap();

        assert_eq!(
            batch.requests(),
            [PromptRequest {
                prompt: "one image".to_string(),
                n: None,
                outputs: vec![PathBuf::from("exact.png")],
            }]
        );
    }

    #[test]
    fn n_zero_is_rejected() {
        let err = PromptBatch::new("none", Path::new("out.png"), &[], None, Some(0))
            .expect_err("zero outputs should fail");

        assert_eq!(err.to_string(), "--n must be greater than 0");
    }

    #[test]
    fn empty_variant_is_rejected() {
        let variants = vec!["".to_string()];

        let err = PromptBatch::new("base", Path::new("out.png"), &variants, None, None)
            .expect_err("empty variants should fail");

        assert_eq!(err.to_string(), "--variant cannot be empty");
    }
}
