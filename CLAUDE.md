# CLAUDE.md — VidGenie

## Project Overview

VidGenie is a headless video generator in Rust. JSON in → MP4 out. Uses OpenGL for rendering and GStreamer for encoding. See `PROJECT_CONTEXT.md` for full architecture details.

## Build & Run

```bash
# Build
make build                    # or: cargo build -p vg-cli

# Run
cargo run -p vg-cli -- --file examples/local_files.json --output output.mp4

# Render examples
make render-example EXAMPLE=local_files.json    # output → outputs/local_files.mp4
make render-all-examples
```

## Test, Lint, Format

```bash
make unit-test    # cargo test --workspace
make lint         # cargo check + cargo clippy --workspace --all-targets
make fmt          # cargo fmt --all
```

## Workspace Layout

```
src/
├── vg-cli/      # Binary entry point. Orchestrates pipeline: JSON → assets → render loop → encode.
│                  Main file: src/vg-cli/src/bin/vidgenie.rs
├── vg-gl/       # OpenGL abstractions (shaders, textures, FBOs, Transformer).
│                  Key: renderer.rs, transformer.rs, texture.rs, framebuffer.rs
├── vg-gst/      # GStreamer encoding. GSVideo + VideoEncoder (mpsc channel to encode thread).
│                  Key: gs_video.rs
└── vg-video/    # Domain logic. Four submodules:
    ├── asset/       # Asset types: Image, Shape, Luma, Video (stub)
    ├── request/     # JSON deserialization: RenderRequest, Clip, Transition, Transform, Caption
    ├── render/      # ImageClipTexture, ShapeClipTexture — asset → GL texture + quad computation
    ├── text/        # Text wrapping (UAX#14), measurement, rasterization via rusttype
    └── overlay/     # CaptionOverlay — pre-rasterized text page textures with anchor positioning
```

## Key Conventions

- **All GL objects implement `Drop`** to release GPU resources — follow this pattern for any new GL types.
- **Transforms are CPU-side**: `Transformer` applies full ortho + model matrix on CPU, writing NDC coords into `Quad`. Vertex shader is pass-through.
- **Luma matte pairing**: A `"luma"` clip must appear directly before its target clip in the JSON clips array. Pairing is done via a `pending_luma` variable during clip iteration.
- **Error handling**: Use `anyhow::bail!` / `anyhow::Context` throughout. No panics in library code.
- **Serde patterns**: `Asset` uses `#[serde(tag = "type")]` (internal tag). `Transition` and `Transform` use `#[serde(untagged)]` for shorthand + detailed forms.
- **One texture unit per clip**, max 32 clips.
- **FPS = 30**, MSAA = 4x.

## Adding a New Asset Type

1. Add variant to `Asset` enum in `src/vg-video/src/asset/asset.rs`
2. Create asset module in `src/vg-video/src/asset/` implementing `MediaAsset` trait
3. Create clip texture struct in `src/vg-video/src/render/` (see `ImageClipTexture` as template)
4. Add `ClipTexture` variant in `src/vg-cli/src/bin/vidgenie.rs`
5. Handle the new type in the clip iteration loop in `vidgenie.rs`

## Adding a New Transform Preset

1. Add variant to `TransformPreset` in `src/vg-video/src/request/transform.rs`
2. Implement keyframe generation in `Transform::to_keyframes()` match arm

## Platform Notes

- **macOS**: Uses `glutin` for GL context. Requires GStreamer from Homebrew.
- **Linux**: Uses `khronos-egl` surfaceless for headless rendering (Docker). Set `VIDGENIE_HEADLESS=1`.
- **Docker**: `make docker-build` then `make docker-render-example EXAMPLE=...`

## Dependencies to Install (macOS)

```bash
brew install gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly
```
