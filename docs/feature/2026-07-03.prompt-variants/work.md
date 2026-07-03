# Work

## Design

Add a prompt batch planning module as the seam between CLI args and transports.

Planner inputs:
- base prompt
- output path
- variants
- variant separator
- `n`

Planner outputs:
- prompt requests, each with final prompt, requested `n`, and concrete output paths.

Output naming:
- one total output: exact `--out`
- `--n N`: `stem-001.ext`
- variants: `stem-001.ext`
- variants plus `--n N`: `stem-001-01.ext`

Transport behavior:
- hosted: flatten planned outputs into independent single-output calls; run concurrently.
- image-api: one request per prompt request; use native `n` for samples and write all returned images.

Separator:
- default newline.
- configured with `--variant-separator`.
- accept any text.
- decode shell-friendly escapes: `\n`, `\t`, `\r`, `\\`; other backslashes stay literal.
