# Caption/Subtitle Feature — Design Plan

## 1. JSON Schema

Add an optional `caption` field to each clip:

```jsonc
{
  "asset": { "type": "image", "src": "..." },
  "start": 0.0,
  "length": 5.0,
  "caption": {
    "text": "Long caption text… 中文混合 English 日本語",
    "lang": "auto",                    // "auto"|"en"|"zh"|"ja"|"ko"|"fr"|"it"
    "font": {
      "src": "./fonts/NotoSansCJK-Regular.ttc",
      "size_px": 42
    },
    "layout": {
      "max_width": { "px": 900 },      // OR { "chars": 26 }
      "max_lines": 2,
      "line_height_mult": 1.2,
      "align": "center"
    },
    "placement": {
      "anchor": "bottom_center",
      "margin_px": { "x": 40, "y": 60 }
    },
    "style": {
      "color": "#FFFFFF",
      "bg_color": "#00000080",
      "padding_px": 12
    },
    "timing": {
      "mode": "even",                  // "even"|"wpm"
      "wpm": 180,
      "min_page_duration": 1.0
    }
  }
}
```

## 2. Text Splitting Algorithm

### Core Approach

Use **Unicode UAX#14 line breaking** (`unicode-linebreak` crate) to find break opportunities. This handles CJK (no spaces), Latin (word boundaries), and mixed text uniformly — no per-language tokenizer needed.

### Algorithm Steps

1. **Normalize input**: trim, collapse repeated whitespace (for Latin), preserve explicit `\n` as hard breaks.
2. **Find legal line-break opportunities** via `unicode-linebreak::linebreaks(text)`, producing text chunks between break points.
3. **Greedy line fitting**: for each chunk, test if appending it to the current line exceeds the constraint:
   - If it fits → append.
   - If not → finalize current line, start a new line with the chunk.
4. **Fallback**: if a single chunk exceeds the constraint (e.g., a long URL), split by individual grapheme clusters.
5. **Group lines into pages** of up to `max_lines` (typical subtitle behavior).
6. **Assign timing** to each page across the clip duration.

### Width Constraint Modes

- **Pixel mode** (`max_width.px`): measure line width using glyph advances from the font.
- **Character mode** (`max_width.chars`): count grapheme clusters (user-perceived characters), not bytes — crucial for CJK where 1 character = 1 grapheme but 3+ bytes.

### Output Structure

```rust
struct CaptionPage {
    start_time: f32,  // seconds relative to clip start
    end_time: f32,
    lines: Vec<String>,
}
```

### Timing Modes

- **`even`**: divide clip duration equally by number of pages (respect `min_page_duration`; warn if too many pages).
- **`wpm`**: estimate duration by grapheme/word count and allocate time proportionally, clamped to clip duration.

## 3. Rendering Strategy — CPU Rasterize → GL Texture → Quad

### Pipeline

1. **At clip load time** (not per frame), for each `CaptionPage`:
   - Rasterize the page text into an `image::RgbaImage` sized to the text box.
   - Upload to an OpenGL `Texture` (same type used for image clips).
2. **Per frame**:
   - Draw the image clip quad (existing behavior).
   - Determine the active caption page by `local_time`.
   - Bind the caption texture and draw an overlay quad on top.

### Why This Works

- Reuses the existing textured-quad pipeline with no shader changes.
- Caption textures are RGBA with alpha, drawn after the image quad with blending already enabled.
- Precomputing textures at load time keeps per-frame cost minimal.

### Integration Point

In `src/vg-cli/src/bin/vidgenie.rs`, after drawing each image clip quad (around line 254–286), draw the caption overlay quad if a caption page is active at the current `local_time`.

### OpenGL Details

- No new shaders required — the existing fragment shader handles textured quads with alpha (`uAlpha` uniform).
- Caption quad positioning uses the same NDC coordinate system, computed from `anchor`, `margin_px`, and the rasterized text dimensions.

## 4. New Module/File Structure

```
src/vg-video/
  request/
    clip.rs              ← add caption: Option<CaptionConfig>
    caption.rs           ← NEW: serde types (CaptionConfig, FontConfig, LayoutConfig, etc.)
  text/
    mod.rs               ← NEW
    split.rs             ← NEW: line breaking + wrapping + paging + timing
    measure.rs           ← NEW: width measurement abstraction (px vs chars)
    rasterize.rs         ← NEW: rasterize lines → RgbaImage
  overlay/
    mod.rs               ← NEW
    caption_overlay.rs   ← NEW: CaptionOverlay struct, manages textures + quad positioning
```

### Core Data Types

```rust
enum MaxWidth {
    Px(u32),
    Chars(u32),
}

enum Lang {
    Auto,
    En,
    Zh,
    Ja,
    Ko,
    Fr,
    It,
}

struct CaptionPage {
    start_time: f32,  // relative to clip start
    end_time: f32,
    lines: Vec<String>,
}

struct CaptionOverlay {
    pages: Vec<CaptionPage>,
    textures: Vec<Texture>,  // one precomputed texture per page
    // + placement/anchor info for quad positioning
}
```

## 5. Rust Crates

| Purpose | Crate | Why |
|---------|-------|-----|
| Unicode line breaking (UAX#14) | `unicode-linebreak` | CJK + mixed text break opportunities — the core of multi-language splitting |
| Grapheme cluster counting | `unicode-segmentation` | Correct "character count" for all languages |
| Font loading + glyph rasterization | `rusttype` | Simple API, kerning support, widely used |
| Image buffer | `image` (already in workspace) | Create `RgbaImage` for caption textures |

## 6. Key Design Decisions

- **No per-language tokenizer**: UAX#14 line breaking handles all 6 languages (+ mixed text) uniformly.
- **CPU rasterization, not GPU text rendering**: simpler, reuses existing quad pipeline, fine for offline rendering.
- **Precompute textures at load time**: one texture per caption page, only bind+draw per frame (cheap).
- **Font requirement**: user must supply a font covering CJK glyphs (e.g., Noto Sans CJK). Fail early with a clear error if glyphs are missing.

## 7. Risks & Mitigations

1. **Font coverage**: CJK requires large font files. Document that users need Noto Sans CJK or similar. Fail early with a clear error if glyphs are missing.
2. **Pixel measurement accuracy**: without full OpenType shaping, minor width discrepancies possible. Mitigate with a small safety margin (subtract 1–2 px from `max_width`).
3. **Too many pages for short clips**: `min_page_duration` clamp + warning log. Optionally merge pages.

## 8. Future / Advanced Path

Consider upgrading if:
- You need high-quality typography (ligatures/kerning/shaping) or scripts beyond the listed set (Arabic/Indic/Thai).
- You need dynamic per-frame text animations or hundreds of captions where per-page textures are too memory-heavy.
- You need automatic font fallback across multiple fonts.

Advanced approach would use `rustybuzz` (HarfBuzz shaping) + `ttf-parser` for proper shaping, a glyph atlas for efficient rendering, and a dedicated text shader with SDF support.
