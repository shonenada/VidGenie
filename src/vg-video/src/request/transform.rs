use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub enum TransformPreset {
    #[serde(rename = "pan_left")]
    PanLeft,
    #[serde(rename = "pan_right")]
    PanRight,
    #[serde(rename = "pan_up")]
    PanUp,
    #[serde(rename = "pan_down")]
    PanDown,
    #[serde(rename = "zoom_in")]
    ZoomIn,
    #[serde(rename = "zoom_out")]
    ZoomOut,
    #[serde(rename = "ken_burns")]
    KenBurns,
    #[serde(rename = "slide_in_left")]
    SlideInLeft,
    #[serde(rename = "slide_in_right")]
    SlideInRight,
    #[serde(rename = "slide_out_left")]
    SlideOutLeft,
    #[serde(rename = "slide_out_right")]
    SlideOutRight,
}

fn default_one() -> f32 {
    1.0
}

fn default_zero() -> f32 {
    0.0
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Keyframe {
    pub time: f32,
    #[serde(default = "default_one")]
    pub scale: f32,
    #[serde(default = "default_zero")]
    pub rotate: f32,
    #[serde(default = "default_zero")]
    pub x: f32,
    #[serde(default = "default_zero")]
    pub y: f32,
    #[serde(default = "default_one")]
    pub opacity: f32,
    #[serde(default = "default_zero")]
    pub skew_x: f32,
    #[serde(default = "default_zero")]
    pub skew_y: f32,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
}

impl Default for Keyframe {
    fn default() -> Self {
        Self {
            time: 0.0,
            scale: 1.0,
            rotate: 0.0,
            x: 0.0,
            y: 0.0,
            opacity: 1.0,
            skew_x: 0.0,
            skew_y: 0.0,
            flip_x: false,
            flip_y: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PresetConfig {
    #[serde(default)]
    pub zoom_start: Option<f32>,
    #[serde(default)]
    pub zoom_end: Option<f32>,
    #[serde(default)]
    pub pan_distance: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Transform {
    Keyframes {
        keyframes: Vec<Keyframe>,
    },
    Preset {
        preset: TransformPreset,
        #[serde(default)]
        preset_config: Option<PresetConfig>,
    },
}

impl Transform {
    pub fn to_keyframes(
        &self,
        clip_length: f32,
        canvas_width: f32,
        canvas_height: f32,
    ) -> Vec<Keyframe> {
        match self {
            Transform::Keyframes { keyframes } => keyframes.clone(),
            Transform::Preset {
                preset,
                preset_config,
            } => preset_to_keyframes(
                *preset,
                preset_config.clone(),
                clip_length,
                canvas_width,
                canvas_height,
            ),
        }
    }
}

fn preset_to_keyframes(
    preset: TransformPreset,
    config: Option<PresetConfig>,
    clip_length: f32,
    canvas_width: f32,
    canvas_height: f32,
) -> Vec<Keyframe> {
    let config = config.unwrap_or_default();
    let pan_distance = config.pan_distance.unwrap_or(100.0);
    let zoom_start = config.zoom_start.unwrap_or(1.0);
    let zoom_end = config.zoom_end.unwrap_or(1.3);

    match preset {
        TransformPreset::PanLeft => vec![
            Keyframe {
                time: 0.0,
                x: 0.0,
                ..Default::default()
            },
            Keyframe {
                time: clip_length,
                x: -pan_distance,
                ..Default::default()
            },
        ],
        TransformPreset::PanRight => vec![
            Keyframe {
                time: 0.0,
                x: 0.0,
                ..Default::default()
            },
            Keyframe {
                time: clip_length,
                x: pan_distance,
                ..Default::default()
            },
        ],
        TransformPreset::PanUp => vec![
            Keyframe {
                time: 0.0,
                y: 0.0,
                ..Default::default()
            },
            Keyframe {
                time: clip_length,
                y: pan_distance,
                ..Default::default()
            },
        ],
        TransformPreset::PanDown => vec![
            Keyframe {
                time: 0.0,
                y: 0.0,
                ..Default::default()
            },
            Keyframe {
                time: clip_length,
                y: -pan_distance,
                ..Default::default()
            },
        ],
        TransformPreset::ZoomIn => vec![
            Keyframe {
                time: 0.0,
                scale: zoom_start,
                ..Default::default()
            },
            Keyframe {
                time: clip_length,
                scale: zoom_end,
                ..Default::default()
            },
        ],
        TransformPreset::ZoomOut => vec![
            Keyframe {
                time: 0.0,
                scale: zoom_end,
                ..Default::default()
            },
            Keyframe {
                time: clip_length,
                scale: zoom_start,
                ..Default::default()
            },
        ],
        TransformPreset::KenBurns => vec![
            Keyframe {
                time: 0.0,
                scale: zoom_start,
                x: 0.0,
                y: 0.0,
                ..Default::default()
            },
            Keyframe {
                time: clip_length,
                scale: zoom_end,
                x: pan_distance * 0.5,
                y: pan_distance * 0.3,
                ..Default::default()
            },
        ],
        TransformPreset::SlideInLeft => vec![
            Keyframe {
                time: 0.0,
                x: -canvas_width,
                opacity: 0.0,
                ..Default::default()
            },
            Keyframe {
                time: 0.5_f32.min(clip_length * 0.2),
                x: 0.0,
                opacity: 1.0,
                ..Default::default()
            },
            Keyframe {
                time: clip_length,
                x: 0.0,
                opacity: 1.0,
                ..Default::default()
            },
        ],
        TransformPreset::SlideInRight => vec![
            Keyframe {
                time: 0.0,
                x: canvas_width,
                opacity: 0.0,
                ..Default::default()
            },
            Keyframe {
                time: 0.5_f32.min(clip_length * 0.2),
                x: 0.0,
                opacity: 1.0,
                ..Default::default()
            },
            Keyframe {
                time: clip_length,
                x: 0.0,
                opacity: 1.0,
                ..Default::default()
            },
        ],
        TransformPreset::SlideOutLeft => vec![
            Keyframe {
                time: 0.0,
                x: 0.0,
                opacity: 1.0,
                ..Default::default()
            },
            Keyframe {
                time: clip_length - 0.5_f32.min(clip_length * 0.2),
                x: 0.0,
                opacity: 1.0,
                ..Default::default()
            },
            Keyframe {
                time: clip_length,
                x: -canvas_width,
                opacity: 0.0,
                ..Default::default()
            },
        ],
        TransformPreset::SlideOutRight => vec![
            Keyframe {
                time: 0.0,
                x: 0.0,
                opacity: 1.0,
                ..Default::default()
            },
            Keyframe {
                time: clip_length - 0.5_f32.min(clip_length * 0.2),
                x: 0.0,
                opacity: 1.0,
                ..Default::default()
            },
            Keyframe {
                time: clip_length,
                x: canvas_width,
                opacity: 0.0,
                ..Default::default()
            },
        ],
    }
}

pub fn interpolate_keyframes(keyframes: &[Keyframe], local_time: f32) -> Keyframe {
    if keyframes.is_empty() {
        return Keyframe::default();
    }
    if keyframes.len() == 1 {
        return keyframes[0];
    }

    let mut sorted: Vec<&Keyframe> = keyframes.iter().collect();
    sorted.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

    if local_time <= sorted[0].time {
        return *sorted[0];
    }
    if local_time >= sorted[sorted.len() - 1].time {
        return *sorted[sorted.len() - 1];
    }

    for i in 0..sorted.len() - 1 {
        let k1 = sorted[i];
        let k2 = sorted[i + 1];
        if local_time >= k1.time && local_time <= k2.time {
            let t = if (k2.time - k1.time).abs() < 0.0001 {
                0.0
            } else {
                (local_time - k1.time) / (k2.time - k1.time)
            };
            return lerp_keyframe(k1, k2, t);
        }
    }

    *sorted[sorted.len() - 1]
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_keyframe(k1: &Keyframe, k2: &Keyframe, t: f32) -> Keyframe {
    Keyframe {
        time: lerp(k1.time, k2.time, t),
        scale: lerp(k1.scale, k2.scale, t),
        rotate: lerp(k1.rotate, k2.rotate, t),
        x: lerp(k1.x, k2.x, t),
        y: lerp(k1.y, k2.y, t),
        opacity: lerp(k1.opacity, k2.opacity, t),
        skew_x: lerp(k1.skew_x, k2.skew_x, t),
        skew_y: lerp(k1.skew_y, k2.skew_y, t),
        flip_x: if t < 0.5 { k1.flip_x } else { k2.flip_x },
        flip_y: if t < 0.5 { k1.flip_y } else { k2.flip_y },
    }
}
