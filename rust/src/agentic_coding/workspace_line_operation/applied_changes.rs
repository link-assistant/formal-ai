//! Apply and report whole-line edits without changing request parsing.
use super::*;

impl LineOperation {
    /// The file after the operation, or `None` when it cannot apply.
    pub(in super::super) fn compute(&self, source: &str) -> Option<String> {
        match self {
            Self::RangeRemoval { .. } | Self::NumberedInsertion { .. } => {
                let (first, last) = self.resolved_lines(source)?;
                match self {
                    Self::NumberedInsertion { text, after, .. } => {
                        inserted_at_line(source, first, text, *after)
                    }
                    _ => removed_range(source, first, last),
                }
            }
            Self::AdjacentRemoval {
                anchor,
                neighbour,
                above,
            } => removed_adjacent(source, anchor, neighbour.as_deref(), *above),
            Self::Move {
                text,
                destination,
                anchor,
            } => moved_line(source, text, *destination, anchor.as_deref()),
            Self::Swap { first, second } => swapped_lines(source, first, second),
        }
    }

    /// Whether the request states the operation's whole extent (a numbered
    /// range), so it may remove most of a file.
    pub(in super::super) const fn bounded(&self) -> bool {
        matches!(self, Self::RangeRemoval { .. })
    }

    /// The seed sentence for the lines actually removed, when the operation
    /// reports them from the file rather than from the request.
    pub(in super::super) fn reported(&self, source: &str) -> Option<(&'static str, Slots)> {
        match self {
            Self::RangeRemoval { from_end: true, .. } => {
                let (first, last) = self.resolved_lines(source)?;
                let lines = if first == last {
                    first.to_string()
                } else {
                    format!("{first}-{last}")
                };
                return Some(("numbered_lines_removed", vec![(LINES_SLOT, lines)]));
            }
            Self::NumberedInsertion {
                text,
                after,
                from_end: true,
                ..
            } => {
                let (line, _) = self.resolved_lines(source)?;
                let intent = if *after {
                    "numbered_line_insert_after"
                } else {
                    "numbered_line_insert_before"
                };
                return Some((
                    intent,
                    vec![(NEW_SLOT, text.clone()), (LINE_SLOT, line.to_string())],
                ));
            }
            _ => {}
        }
        let removed = self.reported_values(source)?;
        Some(("coding_text_remove", vec![(OLD_SLOT, removed.join("\n"))]))
    }

    /// Original removed lines, retained as list values rather than presentation text.
    pub(in super::super) fn reported_values(&self, source: &str) -> Option<Vec<String>> {
        let Self::AdjacentRemoval {
            anchor,
            neighbour,
            above,
        } = self
        else {
            return None;
        };
        let (lines, _) = file_lines(source);
        let (one, two) = adjacent_pair(&lines, anchor, neighbour.as_deref(), *above)?;
        Some(
            [one.min(two), one.max(two)]
                .map(|index| bare(lines[index]).to_owned())
                .to_vec(),
        )
    }
}

impl LineOperation {
    /// The one-based lines a numbered operation names in `source`, earlier
    /// first; `None` for any other operation or a count past the file.
    fn resolved_lines(&self, source: &str) -> Option<(usize, usize)> {
        let count = file_lines(source).0.len();
        let (first, last, from_end) = match self {
            Self::RangeRemoval {
                first,
                last,
                from_end,
            } => (*first, *last, *from_end),
            Self::NumberedInsertion { line, from_end, .. } => (*line, *line, *from_end),
            _ => return None,
        };
        Some((
            resolved_line(first, count, from_end)?,
            resolved_line(last, count, from_end)?,
        ))
    }
}
