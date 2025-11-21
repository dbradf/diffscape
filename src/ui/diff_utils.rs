use ratatui::{
    style::Color,
    text::Span,
};
use similar::{Algorithm, ChangeTag, TextDiff};
use std::ops::Range;

/// Computes the ranges of changes within a line.
/// Returns a tuple of (ranges in old text, ranges in new text) that differ.
pub fn compute_intra_line_diff(old_text: &str, new_text: &str) -> (Vec<Range<usize>>, Vec<Range<usize>>) {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .diff_chars(old_text, new_text);

    let mut old_ranges: Vec<Range<usize>> = Vec::new();
    let mut new_ranges: Vec<Range<usize>> = Vec::new();
    let mut old_idx = 0;
    let mut new_idx = 0;

    for change in diff.iter_all_changes() {
        let len = change.value().len();
        match change.tag() {
            ChangeTag::Equal => {
                old_idx += len;
                new_idx += len;
            }
            ChangeTag::Delete => {
                let range = old_idx..old_idx + len;
                if let Some(last) = old_ranges.last_mut() {
                    if last.end == range.start {
                        last.end = range.end;
                    } else {
                        old_ranges.push(range);
                    }
                } else {
                    old_ranges.push(range);
                }
                old_idx += len;
            }
            ChangeTag::Insert => {
                let range = new_idx..new_idx + len;
                if let Some(last) = new_ranges.last_mut() {
                    if last.end == range.start {
                        last.end = range.end;
                    } else {
                        new_ranges.push(range);
                    }
                } else {
                    new_ranges.push(range);
                }
                new_idx += len;
            }
        }
    }

    (old_ranges, new_ranges)
}

/// Applies diff highlighting to existing syntax highlighted spans.
/// 
/// * `spans` - The original syntax highlighted spans
/// * `diff_ranges` - The ranges that should be highlighted with the diff color
/// * `base_bg` - The background color for the whole line (e.g. dark red for removed)
/// * `highlight_bg` - The background color for the changed parts (e.g. bright red)
pub fn apply_diff_highlight<'a>(
    spans: Vec<Span<'a>>,
    diff_ranges: &[Range<usize>],
    base_bg: Color,
    highlight_bg: Color,
) -> Vec<Span<'a>> {
    if diff_ranges.is_empty() {
        // Just apply the base background to everything
        return spans
            .into_iter()
            .map(|span| {
                let mut style = span.style;
                style = style.bg(base_bg);
                Span::styled(span.content, style)
            })
            .collect();
    }

    let mut new_spans = Vec::new();
    let mut current_idx = 0;

    for span in spans {
        let content = span.content;
        let len = content.len();
        let span_end = current_idx + len;
        let style = span.style;

        let mut last_processed = current_idx;

        // Find ranges that overlap with this span
        for range in diff_ranges {
            // Skip ranges that end before this span
            if range.end <= current_idx {
                continue;
            }
            // Stop if ranges start after this span
            if range.start >= span_end {
                break;
            }

            // Calculate overlap
            let overlap_start = range.start.max(current_idx);
            let overlap_end = range.end.min(span_end);

            // Add non-highlighted part before the overlap
            if overlap_start > last_processed {
                let sub_content = &content[(last_processed - current_idx)..(overlap_start - current_idx)];
                new_spans.push(Span::styled(
                    sub_content.to_string(),
                    style.bg(base_bg),
                ));
            }

            // Add highlighted part
            let sub_content = &content[(overlap_start - current_idx)..(overlap_end - current_idx)];
            new_spans.push(Span::styled(
                sub_content.to_string(),
                style.bg(highlight_bg),
            ));

            last_processed = overlap_end;
        }

        // Add remaining part of the span
        if last_processed < span_end {
            let sub_content = &content[(last_processed - current_idx)..];
            new_spans.push(Span::styled(
                sub_content.to_string(),
                style.bg(base_bg),
            ));
        }

        current_idx += len;
    }

    new_spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_intra_line_diff() {
        let old = "foo bar baz";
        let new = "foo qux baz";
        let (old_ranges, new_ranges) = compute_intra_line_diff(old, new);
        
        // "bar" (4..7) -> "qux" (4..7)
        assert_eq!(old_ranges, vec![4..7]);
        assert_eq!(new_ranges, vec![4..7]);
    }

    #[test]
    fn test_compute_intra_line_diff_multiple() {
        let old = "abc 123 xyz";
        let new = "abc 456 xyz";
        let (old_ranges, new_ranges) = compute_intra_line_diff(old, new);
        
        assert_eq!(old_ranges, vec![4..7]);
        assert_eq!(new_ranges, vec![4..7]);
    }
}
