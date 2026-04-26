# VidGenie — Project Context

## What Is VidGenie?

VidGenie is an **offline, headless video generation tool** written in Rust. It reads a JSON timeline definition and produces an MP4 video by compositing assets frame-by-frame through an OpenGL rendering pipeline, then encoding the pixel stream via GStreamer.

It is designed for **server-side / programmatic video production** — no GUI, no interactive editing. You describe a video as a JSON file and VidGenie renders it to an MP4.

## Core Data Flow

```
JSON file
  → serde deserialize into RenderRequest
  → Load assets (images, shapes) into OpenGL textures
  → Per-frame render loop:
      → For each active clip: compute transform, draw textured quad via OpenGL
      → Resolve MSAA framebuffer → glReadPixels → raw RGB pixels
      → Push Frame to GStreamer encoder thread (mpsc channel)
  → GStreamer pipeline: RGB → I420 → x264enc → H.264 → qtmux → .mp4
```

## Workspace Crates

| Crate | Path | Role |
|---|---|---|
| `vg-cli` | `src/vg-cli/` | CLI binary entry point. Orchestrates the full pipeline: parse JSON, load assets, run render loop, drive encoder. |
| `vg-gl` | `src/vg-gl/` | OpenGL abstractions: context creation, shaders, textures, framebuffers, VAO/VBO/EBO, `Transformer` (matrix math for positioning/scale/rotation/skew). |
| `vg-gst` | `src/vg-gst/` | GStreamer video encoding. `GSVideo` wraps an AppSrc-based pipeline. `VideoEncoder` adds a threaded mpsc channel interface. |
| `vg-video` | `src/vg-video/` | Domain logic: asset types, JSON request structs, clip-to-texture rendering, text wrapping/rasterization, caption overlays. |

## Supported Asset Types

| Type | JSON `"type"` | Status |
|---|---|---|
| Image | `"image"` | Fully implemented. Supports local paths, `file://` URIs, HTTP/HTTPS URLs. |
| Shape | `"shape"` | Fully implemented. Sub-types: `rectangle` (with corner radius), `circle`, `line`. Fill + stroke. |
| Luma Matte | `"luma"` | Fully implemented. Grayscale mask image used as alpha matte for the next clip. |
| Video | `"video"` | **Not implemented** — `todo!()` in `VideoAsset::load()`. |

## Supported Features

- **Positioning**: `position: "center"`, pixel `offset: {x, y}`, `scale`, `rotate`
- **Fade transitions**: Configurable in/out fade with custom duration
- **Keyframe animations**: Explicit keyframes or named presets (`pan_left/right/up/down`, `zoom_in/out`, `ken_burns`, `slide_in/out_left/right`). Interpolated fields: scale, rotate, x, y, opacity, skew_x, skew_y, flip_x, flip_y
- **Luma matte masking**: Luminance-based alpha masking via a dedicated GLSL shader
- **Caption/subtitle overlay**: Full Unicode text wrapping (UAX#14), CJK support, multi-page pagination, CPU-rasterized via rusttype, 9-point anchor positioning, configurable font/size/color/background/padding

## Key Architectural Decisions

1. **CPU-side vertex transforms**: The `Transformer` applies the full ortho-projection + model matrix on the CPU, writing final NDC coordinates into the `Quad`. The vertex shader is a trivial pass-through.
2. **MSAA via framebuffer blit**: Renders to a 4x MSAA FBO, then blits to a resolve FBO for `glReadPixels`.
3. **One texture unit per clip**: Up to 32 clips per render (GL texture unit limit).
4. **Luma matte via ordering convention**: A `"luma"` clip immediately before a content clip in the JSON array is auto-paired as its alpha mask.
5. **Caption textures pre-rasterized**: Text pages are fully rasterized at load time, not per-frame. Runtime cost is just a texture bind + draw call.
6. **Platform-branched GL context**: Linux uses `khronos-egl` surfaceless for headless Docker rendering; macOS uses `glutin`.
7. **Producer-consumer encoding**: The OpenGL render loop (main thread) pushes `Frame`s through an mpsc channel to a GStreamer encode thread.

## Constants

- **FPS**: 30
- **MSAA samples**: 4
- **Max clips**: 32

## Known Limitations

- Video assets not implemented
- No audio support
- Only `"center"` position mode recognized
- WPM timing mode parsed but not used (always "even" timing)
- `vg-video/src/video.rs` and `vg-cli/src/lib.rs` are stubs

## JSON Configuration Structure

```json
{
  "output": { "format": "mp4", "width": 1280, "height": 720 },
  "timeline": {
    "background": "#101418",
    "tracks": [
      {
        "clips": [
          {
            "asset": { "type": "image", "src": "./path/to/image.jpg" },
            "start": 0.0,
            "length": 3.0,
            "offset": { "x": 0, "y": 0 },
            "scale": 1.0,
            "rotate": 0.0,
            "position": "center",
            "transition": "fade",
            "transform": { "preset": "zoom_in" },
            "caption": { "text": "Hello World" }
          }
        ]
      }
    ]
  }
}
```

## Key Dependencies

| Dependency | Role |
|---|---|
| `gl` / `gl_generator` | OpenGL 3.3 core bindings |
| `glutin` / `khronos-egl` | GL context creation (platform-dependent) |
| `nalgebra` / `nalgebra-glm` | Matrix math for transforms |
| `gstreamer` / `gstreamer-app` | Video encoding pipeline |
| `image` | Image decoding and pixel manipulation |
| `reqwest` (blocking) | HTTP image downloading |
| `serde` / `serde_json` | JSON deserialization |
| `rusttype` | Font loading and text rasterization |
| `unicode-linebreak` / `unicode-segmentation` | Text wrapping (UAX#14) and grapheme clustering |
| `colors-transform` | Hex color parsing |
| `clap` | CLI argument parsing |
| `anyhow` / `thiserror` | Error handling |
