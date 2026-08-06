#[derive(Debug, Clone, PartialEq)]
pub enum ParsedRegion {
    Merged { lines: Vec<String> },
    Conflict {
        ours: Vec<String>,
        theirs: Vec<String>,
        base: Option<Vec<String>>,
    },
}

const START: &str = "<<<<<<<";
const BASE: &str = "|||||||";
const SEP: &str = "=======";
const END: &str = ">>>>>>>";

/// Split a conflict-marked file into an ordered list of merged and conflict regions.
/// On malformed/unterminated markers, returns a single manual Conflict region
/// containing the raw text so the caller can still present it for hand editing.
pub fn parse_markers(input: &str) -> Vec<ParsedRegion> {
    let lines: Vec<&str> = input.lines().collect();
    let mut regions: Vec<ParsedRegion> = Vec::new();
    let mut merged: Vec<String> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if starts_with_marker(line, START) {
            // Flush accumulated merged lines.
            if !merged.is_empty() {
                regions.push(ParsedRegion::Merged { lines: std::mem::take(&mut merged) });
            }
            match parse_conflict(&lines, i) {
                Some((region, next)) => {
                    regions.push(region);
                    i = next;
                }
                None => {
                    // Malformed: fall back to one manual region over the whole file.
                    return vec![ParsedRegion::Conflict {
                        ours: input.lines().map(String::from).collect(),
                        theirs: Vec::new(),
                        base: None,
                    }];
                }
            }
        } else {
            merged.push(line.to_string());
            i += 1;
        }
    }
    if !merged.is_empty() {
        regions.push(ParsedRegion::Merged { lines: merged });
    }
    regions
}

fn starts_with_marker(line: &str, marker: &str) -> bool {
    line.starts_with(marker)
}

/// Collect lines starting at `i` until a line matching one of `terminators` is
/// found. If a line matches one of `reject` first, the marker sequence is
/// malformed/out-of-place, so `None` is returned (triggering the caller's
/// fallback). Also returns `None` if EOF is reached before a terminator.
fn collect_until(
    lines: &[&str],
    mut i: usize,
    terminators: &[&str],
    reject: &[&str],
) -> Option<(Vec<String>, usize)> {
    let mut acc: Vec<String> = Vec::new();
    while i < lines.len() && !terminators.iter().any(|t| starts_with_marker(lines[i], t)) {
        if reject.iter().any(|m| starts_with_marker(lines[i], m)) {
            return None; // nested/unexpected marker
        }
        acc.push(lines[i].to_string());
        i += 1;
    }
    if i >= lines.len() {
        return None; // no terminator before EOF
    }
    Some((acc, i))
}

/// Parse one conflict starting at `start` (the `<<<<<<<` line).
/// Returns the region and the index just past the closing `>>>>>>>`.
fn parse_conflict(lines: &[&str], start: usize) -> Option<(ParsedRegion, usize)> {
    let mut base: Option<Vec<String>> = None;

    // ours until BASE or SEP; a stray START/END here is unexpected.
    let (ours, mut i) = collect_until(lines, start + 1, &[BASE, SEP], &[START, END])?;

    // optional base section
    if starts_with_marker(lines[i], BASE) {
        // base until SEP; a stray START/END/BASE here is unexpected.
        let (b, next) = collect_until(lines, i + 1, &[SEP], &[START, END, BASE])?;
        base = Some(b);
        i = next;
    }
    // now on SEP
    if !starts_with_marker(lines[i], SEP) {
        return None;
    }
    i += 1;
    // theirs until END; a stray START/BASE/SEP here is unexpected.
    let (theirs, i) = collect_until(lines, i, &[END], &[START, BASE, SEP])?;

    // lines[i] is END
    Some((ParsedRegion::Conflict { ours, theirs, base }, i + 1))
}
