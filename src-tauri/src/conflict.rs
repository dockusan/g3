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

/// Parse one conflict starting at `start` (the `<<<<<<<` line).
/// Returns the region and the index just past the closing `>>>>>>>`.
fn parse_conflict(lines: &[&str], start: usize) -> Option<(ParsedRegion, usize)> {
    let mut ours: Vec<String> = Vec::new();
    let mut base: Option<Vec<String>> = None;
    let mut theirs: Vec<String> = Vec::new();

    let mut i = start + 1;
    // ours until BASE or SEP
    while i < lines.len()
        && !starts_with_marker(lines[i], BASE)
        && !starts_with_marker(lines[i], SEP)
    {
        if starts_with_marker(lines[i], START) || starts_with_marker(lines[i], END) {
            return None; // nested/unexpected marker
        }
        ours.push(lines[i].to_string());
        i += 1;
    }
    if i >= lines.len() {
        return None;
    }
    // optional base section
    if starts_with_marker(lines[i], BASE) {
        i += 1;
        let mut b: Vec<String> = Vec::new();
        while i < lines.len() && !starts_with_marker(lines[i], SEP) {
            b.push(lines[i].to_string());
            i += 1;
        }
        if i >= lines.len() {
            return None;
        }
        base = Some(b);
    }
    // now on SEP
    if !starts_with_marker(lines[i], SEP) {
        return None;
    }
    i += 1;
    // theirs until END
    while i < lines.len() && !starts_with_marker(lines[i], END) {
        theirs.push(lines[i].to_string());
        i += 1;
    }
    if i >= lines.len() {
        return None; // no closing marker
    }
    // lines[i] is END
    Some((ParsedRegion::Conflict { ours, theirs, base }, i + 1))
}
