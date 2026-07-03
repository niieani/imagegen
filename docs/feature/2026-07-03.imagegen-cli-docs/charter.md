# imagegen CLI docs update

## Brief

Update local `imagegen` CLI help and bundled `imagegen-cli` skill with current OpenAI Image API guidance from the official image generation guide.

## Goal

Users and agents can discover, from `imagegen --help`, subcommand help, and the skill docs:

- `gpt-image-2` is the default model.
- `edit` uses input images as edit targets or references; prompts must state change vs preserve constraints.
- `gpt-image-2` size constraints and popular dimensions are visible.
- `gpt-image-2` does not support `background=transparent`.
- High-res / complex work can take around 2 minutes; `quality=low` and JPEG are latency levers in the API, though this CLI writes PNG.
- `gpt-image-2` can generate newspaper/screenshot-like images with ALL prompt-provided text and is very likely to render that text correctly.

## Scope

In:

- `src/args.rs` help text.
- `skills/imagegen-cli/SKILL.md`.
- New skill reference doc if useful.
- `README.md` alignment.
- CLI parsing/help tests.

Out:

- Changing auth or transport behavior.
- Changing output format support.

## Decisions

- Keep the top-level skill concise; put detailed model/output facts behind a reference pointer.
- Use official OpenAI docs as source of truth; upstream sample skill may inform structure only.

## Verification

- `cargo test` must pass.
- `cargo fmt` must pass.
- Help tests assert important user-visible strings.
- Manual diff review confirms the docs focus on supported workflow and model limits.
