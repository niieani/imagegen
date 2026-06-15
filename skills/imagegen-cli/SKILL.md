---
name: imagegen-cli
description: Generates or edits raster images through the local imagegen CLI, reusing the user's Codex authentication and writing image files to disk. Use when the user asks to create, generate, edit, modify, transform, or produce images with imagegen or the local imagegen command.
---

# Imagegen CLI

## Quick Start

If `imagegen` is missing, stop and tell the user it must be installed in `PATH`
and authenticated first. If a generation or edit command fails because Codex
ChatGPT/backend authentication is missing, stop and tell the user to run
`codex login`.

Generate a new image:

```sh
imagegen generate \
  --prompt "a small blue ceramic teapot on a white table" \
  --out teapot.png
```

Edit one or more images, or use them as references for a new output:

```sh
imagegen edit \
  --image input.png \
  --prompt "use the input as a reference and create a matching product photo" \
  --background transparent \
  --out output.png
```

## Workflow

1. Clarify only missing creative constraints that materially affect the result;
   otherwise choose sensible defaults and proceed.
2. Use an explicit `.png` output path. Prefer the user's requested path;
   otherwise write under the active workspace, using its scratch/output
   convention when known.
3. Run `imagegen generate` for text-to-image requests and `imagegen edit` for
   transformations of existing images or for using input images as references.
   For edits, make the prompt specify what to do with the inputs.
4. After success, report the output path. In Codex desktop, show the image with
   Markdown using an absolute filesystem path when useful.

## Commands And Options

Both `generate` and `edit` support:

- `--prompt <PROMPT>`: required. Make prompts specific about subject, style,
  composition, background, text handling, and output constraints.
- `--out <PATH>`: required. Must be a `.png` file. The CLI creates parent
  directories as needed.
- `--model <MODEL>`: defaults to `gpt-image-2`.
- `--background <auto|opaque|transparent>`: defaults to `auto`.
- `--quality <auto|low|medium|high>`: defaults to `auto`; use `low` for drafts.
- `--size <SIZE>`: defaults to `auto`; pass the user's requested size exactly.
- `--n <N>`: omit unless requested. `codex-hosted` supports only one output.
- `--transport <codex-hosted|image-api>`: use the default unless the user or
  provider setup requires a specific transport.
- `--codex-home <DIR>`: use only when the user specifies a nonstandard Codex
  home or the environment requires it.

`edit` additionally requires one to five `--image <PATH>` inputs. Provide the
flag multiple times to pass multiple images. Supported input extensions are
`.png`, `.jpg`, `.jpeg`, and `.webp`.

## Failure Handling

- Missing binary: tell the user `imagegen` must be installed in `PATH` and that
  the agent cannot continue until it is available.
- Missing auth: tell the user `imagegen` must be authenticated via Codex
  (`codex login`) and that the agent cannot continue until authentication works.
- Unsupported input format: ask for or convert to PNG/JPEG/WebP only if a normal
  local conversion tool is already available.
- `codex-hosted` with `--n` greater than 1: rerun with one output or switch to
  `--transport image-api` only when the provider supports it.
