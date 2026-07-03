# Prompt Variants

## Brief

Support intentional image batches from one base prompt plus repeated variants. The base prompt is concatenated directly with each variant using a configurable separator. Default separator is a newline.

## Goal

`imagegen generate` and `imagegen edit` accept repeated `--variant <TEXT>`. Each variant produces an image request whose prompt is:

```text
<prompt><variant-separator><variant>
```

`--variant-separator <TEXT>` configures the separator; default behavior is newline. No `"Variant:"` label is injected.

## Scope

In scope:
- CLI args for prompt variants and separator.
- Shared batch planner for prompt/output mapping.
- Hosted default auth support by parallelizing single-output hosted calls.
- Direct image API writing all returned images instead of only the first.
- README/help updates and tests.

Out of scope:
- Prompt-file batch mode.
- Queue persistence/retry.
- Changing auth/provider resolution.

## Criteria And Verification

- Variants append exactly via configured separator; verified by unit tests.
- `--n` and `--variant` produce deterministic suffixed output paths; verified by unit tests.
- `--n 0` hard-fails; verified by unit tests.
- Hosted transport no longer rejects `--n > 1` at CLI orchestration; verified by unit tests where feasible and compile-time flow.
- Direct image API can write multiple returned images; verified by unit tests around request/output mapping where local.
- Code formatted and project tests pass with `cargo fmt` and `cargo test`.

## Execution Shape

Small direct feature. Use TDD for planner semantics first, then wire transports.
