use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    Auto,
    En,
    Zh,
    Ja,
    Ko,
    Fr,
    It,
}

#[derive(Debug, Deserialize)]
pub struct FontConfig {
    pub src: String,
    #[serde(default = "default_size_px")]
    pub size_px: u32,
}

fn default_size_px() -> u32 { 36 }

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MaxWidth {
    Px { px: u32 },
    Chars { chars: u32 },
}

fn default_max_lines() -> u32 { 2 }

fn default_line_height_mult() -> f32 { 1.3 }

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl Default for TextAlign {
    fn default() -> Self { TextAlign::Center }
}

#[derive(Debug, Deserialize)]
pub struct LayoutConfig {
    #[serde(default)]
    pub max_width: Option<MaxWidth>,
    #[serde(default = "default_max_lines")]
    pub max_lines: u32,
    #[serde(default = "default_line_height_mult")]
    pub line_height_mult: f32,
    #[serde(default)]
    pub align: TextAlign,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnchorPosition {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Default for AnchorPosition {
    fn default() -> Self { AnchorPosition::BottomCenter }
}

fn default_margin_x() -> f32 { 40.0 }

fn default_margin_y() -> f32 { 60.0 }

#[derive(Debug, Deserialize)]
pub struct MarginPx {
    #[serde(default = "default_margin_x")]
    pub x: f32,
    #[serde(default = "default_margin_y")]
    pub y: f32,
}

#[derive(Debug, Deserialize)]
pub struct PlacementConfig {
    #[serde(default)]
    pub anchor: AnchorPosition,
    #[serde(default)]
    pub margin_px: Option<MarginPx>,
}

fn default_color() -> String { "#FFFFFF".to_string() }

fn default_padding_px() -> f32 { 8.0 }

#[derive(Debug, Deserialize)]
pub struct CaptionStyle {
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default)]
    pub bg_color: Option<String>,
    #[serde(default = "default_padding_px")]
    pub padding_px: f32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimingMode {
    Even,
    Wpm,
}

impl Default for TimingMode {
    fn default() -> Self { TimingMode::Even }
}

fn default_wpm() -> u32 { 180 }

fn default_min_page_duration() -> f32 { 1.0 }

#[derive(Debug, Deserialize)]
pub struct TimingConfig {
    #[serde(default)]
    pub mode: TimingMode,
    #[serde(default = "default_wpm")]
    pub wpm: u32,
    #[serde(default = "default_min_page_duration")]
    pub min_page_duration: f32,
}

#[derive(Debug, Deserialize)]
pub struct CaptionConfig {
    pub text: String,
    #[serde(default)]
    pub lang: Option<Lang>,
    #[serde(default)]
    pub font: Option<FontConfig>,
    #[serde(default)]
    pub layout: Option<LayoutConfig>,
    #[serde(default)]
    pub placement: Option<PlacementConfig>,
    #[serde(default)]
    pub style: Option<CaptionStyle>,
    #[serde(default)]
    pub timing: Option<TimingConfig>,
}
