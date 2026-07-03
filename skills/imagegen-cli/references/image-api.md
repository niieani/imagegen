# Image API Reference

Source: https://developers.openai.com/api/docs/guides/image-generation

Use this for model behavior, size choices, latency, and edit prompting.

## CLI Surface

Local `imagegen` commands:

- `generate`: `--prompt`, `--out`, `--model`, `--background`, `--quality`,
  `--size`, `--n`, `--variant`, `--variant-separator`, `--transport`
- `edit`: same controls plus one to five repeated `--image` inputs
- PNG output only

## gpt-image-2 Defaults

Use `gpt-image-2` for new generation/edit work unless the user gives another
model.

- Strengths: photorealism, compositing, high-fidelity edits, dense text in
  images, UI screenshots, posters, product labels, newspapers, diagrams.
- Text prompts: `gpt-image-2` can generate a newspaper page or screenshot with
  ALL text provided in the prompt and is very likely to render that text
  correctly. Quote exact strings, specify typography and placement, and use
  medium/high quality for small text or dense multi-font layouts.
- Limitation: `background=transparent` is not supported.
- Latency: complex or high-resolution work can take up to about 2 minutes. 4K
  output is a normal place to warn the user before starting batches.

## Customize Output

CLI controls:

- `size`: dimensions
- `quality`: `low`, `medium`, `high`, `auto`
- `background`: `auto`, `opaque`, `transparent`

Use `quality=low` for fast drafts, thumbnails, and quick iteration. Use
`medium` or `high` for final assets, dense text, diagrams, identity-sensitive
edits, and high-resolution output.

## gpt-image-2 Sizes

Popular sizes:

| Use | Size |
| --- | --- |
| Square | `1024x1024` |
| Landscape | `1536x1024` |
| Portrait | `1024x1536` |
| 2K square | `2048x2048` |
| 2K landscape | `2048x1152` |
| 4K landscape | `3840x2160` |
| 4K portrait | `2160x3840` |
| Default | `auto` |

Custom `gpt-image-2` sizes must satisfy all constraints:

- Maximum edge length <= `3840px`.
- Both edges are multiples of `16px`.
- Long edge to short edge ratio <= `3:1`.
- Total pixels between `655,360` and `8,294,400`.

Square images are usually fastest. Outputs above `2560x1440` total pixels are
experimental per the guide.

## Edit Workflows

`imagegen edit` can:

- Modify existing images.
- Generate a new image using one or more images as references.

For local `imagegen edit`, pass one to five repeated `--image` flags. The order
matters. In the prompt, name each input by index and role:

```sh
imagegen edit \
  --image product.png \
  --image lighting-reference.png \
  --prompt "Image 1 is the product. Image 2 is the lighting reference. Create a product photo of Image 1 using the lighting style of Image 2. Keep the label text unchanged." \
  --quality high \
  --out product-lit.png
```

For surgical edits, state both sides:

- Change only the target detail.
- Preserve identity, geometry, labels, camera angle, background, lighting, or
  other invariants that matter.
