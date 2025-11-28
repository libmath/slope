use crate::error::*;

use std::path::Path;

/// Take in a line with an index number, and checks if it starts with a "--".
fn no_comment(enumerated_line: &(usize, &str)) -> bool {
    !enumerated_line.1.trim_start().starts_with("--")
}

/// If the line contains any of the strings given as `starters`, then it MUST
/// start with that string.
pub fn line_starts_with(
    path: &Path,
    text: &str,
    starters: &[&str],
) -> Result<()> {
    for (line_idx, line) in text.lines().enumerate().filter(no_comment) {
        for s in starters {
            if line.contains(s) && !line.starts_with(s) {
                return Err(Error::Content {
                    filepath: path.to_path_buf(),
                    line_idx,
                    err: LineError::StartsWith(s.to_string()),
                });
            }
        }
    }
    Ok(())
}

pub fn space_between_import_libraries(path: &Path, text: &str) -> Result<()> {
    let mut last_mathlib = None;

    for (line_idx, line) in text
        .lines()
        .enumerate()
        .filter_map(|(i, v)| Some((i, v.strip_prefix("import")?)))
    {
        let line = line.trim_start();
        if line.starts_with("Mathlib") {
            last_mathlib = Some(line_idx);
            continue;
        }
        let Some(pidx) = last_mathlib else { continue };
        let bad = line.starts_with("Dino") && pidx + 1 == line_idx;
        if bad {
            return Err(Error::Content {
                filepath: path.to_path_buf(),
                line_idx,
                err: LineError::LeaveSpace,
            });
        }
    }
    Ok(())
}

pub fn strip_trailing_whitespace(text: &str) -> String {
    let lines = text.lines().map(|v| v.trim_end()).collect::<Vec<_>>();
    lines.join("\n") + "\n"
}
