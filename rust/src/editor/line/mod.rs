use crate::editor::{AnnotatedString, Annotation};
use crate::prelude::{ByteIdx, ColIdx, GraphemeIdx};
use grapheme_width::GraphemeWidth;
use std::cmp::min;
use std::fmt::{Display, Formatter, Result};
use std::ops::{Deref, Range};
use unicode_segmentation::UnicodeSegmentation;
mod grapheme_width;
mod text_fragment;
use text_fragment::TextFragment;

#[derive(Default, Clone)]
pub struct Line {
    fragments: Vec<TextFragment>,
    pub string: String,
}

impl Line {
    pub fn from(string: &str) -> Self {
        debug_assert!(string.is_empty() || string.lines().count() == 1);
        Self {
            fragments: Self::string_to_fragments(string),
            string: String::from(string),
        }
    }
    fn string_to_fragments(string: &str) -> Vec<TextFragment> {
        string
            .grapheme_indices(true)
            .map(|(start_byte_idx, grapheme)| TextFragment::new(start_byte_idx, grapheme))
            .collect()
    }
    fn rebuild_fragments(&mut self) {
        self.fragments = Self::string_to_fragments(&self.string);
    }
    #[allow(dead_code)]
    pub fn get_visible_graphemes(&self, range: Range<ColIdx>) -> String {
        self.get_annotated_visible_substr(range, None).to_string()
    }
    /// Get the annotated string in the given column index.
    /// Note that the column index is the same as the grapheme index.
    /// A grapheme can have a width of 2 columns.
    /// Parameters:
    /// - range: the range of columns to get the annotated string from.
    /// - query: the query to highlight in the annotated string.
    /// - selected_match: the selected match to highlight in the annotated string. this is
    ///   only applied if the query is not empty.
    pub fn get_annotated_visible_substr(
        &self,
        range: Range<ColIdx>,
        annotations: Option<&Vec<Annotation>>,
    ) -> AnnotatedString {
        if range.start >= range.end {
            return AnnotatedString::default();
        }
        let mut result = AnnotatedString::from(&self.string);

        if let Some(annotations) = annotations {
            for annotation in annotations {
                let start_byte_idx = annotation.start_byte_idx;
                let end_byte_idx = annotation.end_byte_idx;
                let annotation_type = annotation.annotation_type;
                result.push(annotation_type, start_byte_idx, end_byte_idx);
            }
        }

        // Insert replacement characters, and truncate if needed.
        // We do this backwards, otherwise the byte indices would be off in case
        // a replacement character has a different width than the original character.

        // start from far right column
        let mut fragment_start = self.width();

        for fragment in self.fragments.iter().rev() {
            let fragment_end = fragment_start;
            fragment_start = fragment_start.saturating_sub(fragment.width.as_usize());
            if fragment_start > range.end {
                // skip if we are not in the range
                continue;
            }

            if fragment_start < range.end && range.end < fragment_end {
                // clip right if the fragment is partially visible
                result.replace(fragment.start_byte_idx, self.string.len(), "⋯");
                continue;
            } else if fragment_start == range.end {
                // truncate right if we have reached the end of the visible range
                result.truncate_right_from(fragment.start_byte_idx);
                continue;
            }

            // fragments is in the visible range below

            if fragment_end <= range.start {
                // fragment ends at the start of the range:
                // remove the entire left side of the string
                result.truncate_left_until(
                    fragment
                        .start_byte_idx
                        .saturating_add(fragment.grapheme.len()),
                );
                // end process since all remaining fragments will be invisible
                break;
            } else if fragment_start < range.start && range.start < fragment_end {
                // fragment overlaps with the start of the range:
                // remove the left side of the fragment and add ellipses
                result.replace(
                    0,
                    fragment
                        .start_byte_idx
                        .saturating_add(fragment.grapheme.len()),
                    "⋯",
                );
                // end process since all remaining fragments will be invisible
                break;
            }

            // fragments is fully within the range below
            if range.start <= fragment_start && fragment_end <= range.end {
                if let Some(replacement) = fragment.replacement {
                    let start_byte_idx = fragment.start_byte_idx;
                    let end_byte_idx = start_byte_idx.saturating_add(fragment.grapheme.len());
                    result.replace(start_byte_idx, end_byte_idx, &replacement.to_string());
                }
            }
        }

        result
    }
    pub fn grapheme_count(&self) -> GraphemeIdx {
        self.fragments.len()
    }
    pub fn width_until(&self, grapheme_idx: GraphemeIdx) -> ColIdx {
        self.fragments
            .iter()
            .take(grapheme_idx)
            .map(|fragment| fragment.width.as_usize())
            .sum()
    }
    pub fn width(&self) -> ColIdx {
        self.width_until(self.grapheme_count())
    }

    pub fn insert(&mut self, at: GraphemeIdx, string: &str) {
        debug_assert!(at.saturating_sub(1) <= self.grapheme_count());
        if let Some(fragment) = self.fragments.get(at) {
            self.string.insert_str(fragment.start_byte_idx, string);
        } else {
            self.string.push_str(string);
        }
        self.rebuild_fragments();
    }
    pub fn remove(&mut self, start: GraphemeIdx, length: GraphemeIdx) {
        debug_assert!(start <= self.grapheme_count());
        if let Some(start_fragment) = self.fragments.get(start) {
            let end = start.saturating_add(length);
            let start_byte_idx = start_fragment.start_byte_idx;
            if let Some(end_fragment) = self.fragments.get(end) {
                let end_byte_idx = end_fragment.start_byte_idx;
                self.string.drain(start_byte_idx..end_byte_idx);
            } else {
                self.string.drain(start_byte_idx..);
            }
            self.rebuild_fragments();
        }
    }
    pub fn append(&mut self, other: &Self) {
        self.insert(self.len(), &other.string);
    }

    pub fn split_off(&mut self, at: GraphemeIdx) -> Self {
        if let Some(fragment) = self.fragments.get(at) {
            let remainder = self.string.split_off(fragment.start_byte_idx);
            self.rebuild_fragments();
            Self::from(&remainder)
        } else {
            Self::default()
        }
    }
    fn byte_idx_to_grapheme_idx(&self, byte_idx: ByteIdx) -> Option<GraphemeIdx> {
        if byte_idx > self.string.len() {
            return None;
        }
        self.fragments
            .iter()
            .position(|fragment| fragment.start_byte_idx >= byte_idx)
    }
    fn grapheme_idx_to_byte_idx(&self, grapheme_idx: GraphemeIdx) -> ByteIdx {
        debug_assert!(grapheme_idx <= self.grapheme_count());
        if grapheme_idx == 0 || self.grapheme_count() == 0 {
            return 0;
        }
        self.fragments.get(grapheme_idx).map_or_else(
            || {
                #[cfg(debug_assertions)]
                panic!("grapheme_idx_to_byte_idx: grapheme_idx: {grapheme_idx:?} not found");
                #[cfg(not(debug_assertions))]
                0
            },
            |fragment| fragment.start_byte_idx,
        )
    }
    pub fn search_forward(
        &self,
        query: &str,
        from_grapheme_idx: GraphemeIdx,
    ) -> Option<GraphemeIdx> {
        debug_assert!(from_grapheme_idx <= self.grapheme_count());
        if from_grapheme_idx == self.grapheme_count() {
            return None;
        }
        let start_byte_idx = self.grapheme_idx_to_byte_idx(from_grapheme_idx);
        self.find_all(query, start_byte_idx..self.string.len())
            .first()
            .map(|(_, grapheme_idx)| *grapheme_idx)
    }
    pub fn search_backward(
        &self,
        query: &str,
        from_grapheme_idx: GraphemeIdx,
    ) -> Option<GraphemeIdx> {
        debug_assert!(from_grapheme_idx <= self.grapheme_count());
        if from_grapheme_idx == 0 {
            return None;
        }
        let end_byte_idx = if from_grapheme_idx == self.grapheme_count() {
            self.string.len()
        } else {
            self.grapheme_idx_to_byte_idx(from_grapheme_idx)
        };
        self.find_all(query, 0..end_byte_idx)
            .last()
            .map(|(_, grapheme_idx)| *grapheme_idx)
    }
    pub fn find_all(&self, query: &str, range: Range<ByteIdx>) -> Vec<(ByteIdx, GraphemeIdx)> {
        let start_byte_idx = range.start;
        let end_byte_idx = min(range.end, self.string.len());
        debug_assert!(start_byte_idx <= end_byte_idx);
        debug_assert!(start_byte_idx <= self.string.len());
        self.string
            .get(start_byte_idx..end_byte_idx)
            .map_or_else(Vec::new, |substr| {
                let potential_matches: Vec<ByteIdx> = substr
                    .match_indices(query)
                    .map(|(relative_start_idx, _)| {
                        relative_start_idx.saturating_add(start_byte_idx)
                    })
                    .collect();
                self.match_grapheme_clusters(&potential_matches, query)
            })
    }
    fn match_grapheme_clusters(
        &self,
        matches: &[ByteIdx],
        query: &str,
    ) -> Vec<(ByteIdx, GraphemeIdx)> {
        let grapheme_count = query.graphemes(true).count();
        matches
            .iter()
            .filter_map(|&start_byte_idx| {
                self.byte_idx_to_grapheme_idx(start_byte_idx)
                    .and_then(|grapheme_idx| {
                        self.fragments
                            .get(grapheme_idx..grapheme_idx.saturating_add(grapheme_count))
                            .and_then(|fragments| {
                                let substring = fragments
                                    .iter()
                                    .map(|fragment| fragment.grapheme.as_str())
                                    .collect::<String>();
                                (substring == query).then_some((start_byte_idx, grapheme_idx))
                            })
                    })
            })
            .collect()
    }
}

impl Display for Line {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        write!(formatter, "{}", self.string)
    }
}
impl Deref for Line {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.string
    }
}
