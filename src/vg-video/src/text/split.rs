use unicode_linebreak::{linebreaks, BreakOpportunity};
use unicode_segmentation::UnicodeSegmentation;

use crate::text::measure::WidthMode;

#[derive(Debug, Clone)]
pub struct CaptionPage {
    pub start_time: f32,
    pub end_time: f32,
    pub lines: Vec<String>,
}

pub fn split_text_into_pages(
    text: &str,
    max_width: f32,
    max_lines: u32,
    clip_length: f32,
    min_page_duration: f32,
    mode: &WidthMode,
) -> Vec<CaptionPage> {
    if text.is_empty() {
        return Vec::new();
    }

    let wrapped_lines = wrap_text(text, max_width, mode);
    if wrapped_lines.is_empty() {
        return Vec::new();
    }

    let pages = group_into_pages(&wrapped_lines, max_lines);
    assign_timings(pages, clip_length, min_page_duration)
}

fn wrap_text(text: &str, max_width: f32, mode: &WidthMode) -> Vec<String> {
    let mut lines = Vec::new();

    for segment in text.split('\n') {
        if segment.is_empty() {
            lines.push(String::new());
            continue;
        }

        let chunks = extract_chunks(segment);
        let mut current_line = String::new();

        for chunk in &chunks {
            if current_line.is_empty() {
                let trimmed = chunk.trim_start();
                if mode.measure(trimmed) <= max_width {
                    current_line = trimmed.to_string();
                } else {
                    let mut split_lines = split_by_graphemes(trimmed, max_width, mode);
                    if let Some(last) = split_lines.pop() {
                        for l in split_lines {
                            lines.push(l.trim_end().to_string());
                        }
                        current_line = last;
                    }
                }
            } else {
                let candidate = format!("{}{}", current_line, chunk);
                if mode.measure(&candidate) <= max_width {
                    current_line = candidate;
                } else {
                    lines.push(current_line.trim_end().to_string());
                    let trimmed = chunk.trim_start();
                    if mode.measure(trimmed) <= max_width {
                        current_line = trimmed.to_string();
                    } else {
                        let mut split_lines = split_by_graphemes(trimmed, max_width, mode);
                        if let Some(last) = split_lines.pop() {
                            for l in split_lines {
                                lines.push(l.trim_end().to_string());
                            }
                            current_line = last;
                        } else {
                            current_line = String::new();
                        }
                    }
                }
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line.trim_end().to_string());
        }
    }

    lines
}

fn extract_chunks(segment: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut prev = 0;

    for (offset, opportunity) in linebreaks(segment) {
        if opportunity == BreakOpportunity::Mandatory || opportunity == BreakOpportunity::Allowed {
            if offset > prev {
                chunks.push(segment[prev..offset].to_string());
            }
            prev = offset;
        }
    }

    if prev < segment.len() {
        chunks.push(segment[prev..].to_string());
    }

    chunks
}

fn split_by_graphemes(text: &str, max_width: f32, mode: &WidthMode) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for g in text.graphemes(true) {
        let candidate = format!("{}{}", current, g);
        if mode.measure(&candidate) > max_width && !current.is_empty() {
            lines.push(current);
            current = g.to_string();
        } else {
            current = candidate;
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

fn group_into_pages(lines: &[String], max_lines: u32) -> Vec<Vec<String>> {
    lines
        .chunks(max_lines as usize)
        .map(|chunk| chunk.to_vec())
        .collect()
}

fn assign_timings(
    pages: Vec<Vec<String>>,
    clip_length: f32,
    min_page_duration: f32,
) -> Vec<CaptionPage> {
    let n = pages.len() as f32;
    if n == 0.0 {
        return Vec::new();
    }

    let raw_duration = clip_length / n;
    let page_duration = raw_duration.max(min_page_duration);

    pages
        .into_iter()
        .enumerate()
        .filter_map(|(i, lines)| {
            let start_time = i as f32 * page_duration;
            if start_time >= clip_length {
                return None;
            }
            let end_time = (start_time + page_duration).min(clip_length);
            Some(CaptionPage {
                start_time,
                end_time,
                lines,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars_mode() -> WidthMode {
        WidthMode::chars_mode()
    }

    #[test]
    fn empty_text() {
        let result = split_text_into_pages("", 20.0, 2, 10.0, 1.0, &chars_mode());
        assert!(result.is_empty());
    }

    #[test]
    fn single_line_fits() {
        let result = split_text_into_pages("hello", 20.0, 2, 10.0, 1.0, &chars_mode());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].lines, vec!["hello"]);
        assert_eq!(result[0].start_time, 0.0);
        assert_eq!(result[0].end_time, 10.0);
    }

    #[test]
    fn wraps_long_line() {
        let result =
            split_text_into_pages("hello world foo bar", 10.0, 2, 10.0, 1.0, &chars_mode());
        assert!(result.len() >= 1);
        for page in &result {
            assert!(page.lines.len() <= 2);
            for line in &page.lines {
                assert!(line.graphemes(true).count() <= 10, "line too long: {}", line);
            }
        }
    }

    #[test]
    fn multiple_pages() {
        let text = "aaa bbb ccc ddd eee fff ggg hhh";
        let result = split_text_into_pages(text, 8.0, 1, 20.0, 1.0, &chars_mode());
        assert!(result.len() > 1);
        for page in &result {
            assert!(page.lines.len() <= 1);
        }
    }

    #[test]
    fn hard_line_break() {
        let result = split_text_into_pages("line1\nline2", 20.0, 2, 10.0, 1.0, &chars_mode());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].lines, vec!["line1", "line2"]);
    }

    #[test]
    fn respects_min_page_duration() {
        let result = split_text_into_pages(
            "a\nb\nc\nd\ne\nf",
            20.0,
            1,
            6.0,
            2.0,
            &chars_mode(),
        );
        for page in &result {
            let duration = page.end_time - page.start_time;
            assert!(
                duration >= 2.0 - f32::EPSILON,
                "page duration {} is less than min 2.0",
                duration
            );
        }
    }

    #[test]
    fn trims_trailing_whitespace() {
        let result =
            split_text_into_pages("hello world this is long", 10.0, 2, 10.0, 1.0, &chars_mode());
        for page in &result {
            for line in &page.lines {
                assert_eq!(line, line.trim_end(), "trailing whitespace in: {:?}", line);
            }
        }
    }

    #[test]
    fn grapheme_fallback_split() {
        let text = "abcdefghijklmnopqrstuvwxyz";
        let result = split_text_into_pages(text, 10.0, 2, 10.0, 1.0, &chars_mode());
        assert!(result.len() >= 1);
        for page in &result {
            for line in &page.lines {
                assert!(
                    line.graphemes(true).count() <= 10,
                    "line too long after grapheme split: {}",
                    line
                );
            }
        }
    }
}
