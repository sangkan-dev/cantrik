use similar::{ChangeTag, TextDiff};

pub fn changed_new_line_indices(old_text: &str, new_text: &str) -> Vec<usize> {
    let diff = TextDiff::from_lines(old_text, new_text);
    let mut out = Vec::new();
    let mut new_line: usize = 0;
    for change in diff.iter_all_changes() {
        let tag = change.tag();
        let value = change.value();
        // `similar` chunks often end with `\n`; count logical lines without double-counting the terminator.
        let line_count = if value.is_empty() {
            0
        } else if value.ends_with('\n') {
            value.matches('\n').count().max(1)
        } else {
            value.matches('\n').count() + 1
        };
        match tag {
            ChangeTag::Equal => {
                new_line += line_count;
            }
            ChangeTag::Delete => {}
            ChangeTag::Insert => {
                for i in 0..line_count {
                    out.push(new_line + i);
                }
                new_line += line_count;
            }
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_insert_marks_new_lines() {
        let old = "a\nb\n";
        let new = "a\nx\nb\n";
        let ch = changed_new_line_indices(old, new);
        assert!(ch.contains(&1));
    }
}
