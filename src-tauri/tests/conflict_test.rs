use tauri_app_lib::conflict::{parse_markers, ParsedRegion};

#[test]
fn parses_single_two_way_conflict() {
    let input = "\
line before
<<<<<<< HEAD
our change
=======
their change
>>>>>>> branch
line after
";
    let regions = parse_markers(input);
    assert_eq!(regions.len(), 3);

    match &regions[0] {
        ParsedRegion::Merged { lines } => assert_eq!(lines, &vec!["line before".to_string()]),
        _ => panic!("expected merged"),
    }
    match &regions[1] {
        ParsedRegion::Conflict { ours, theirs, base } => {
            assert_eq!(ours, &vec!["our change".to_string()]);
            assert_eq!(theirs, &vec!["their change".to_string()]);
            assert_eq!(base, &None);
        }
        _ => panic!("expected conflict"),
    }
    match &regions[2] {
        ParsedRegion::Merged { lines } => assert_eq!(lines, &vec!["line after".to_string()]),
        _ => panic!("expected merged"),
    }
}

#[test]
fn parses_diff3_conflict_with_base() {
    let input = "\
<<<<<<< HEAD
ours
||||||| base
original
=======
theirs
>>>>>>> branch
";
    let regions = parse_markers(input);
    assert_eq!(regions.len(), 1);
    match &regions[0] {
        ParsedRegion::Conflict { ours, theirs, base } => {
            assert_eq!(ours, &vec!["ours".to_string()]);
            assert_eq!(theirs, &vec!["theirs".to_string()]);
            assert_eq!(base, &Some(vec!["original".to_string()]));
        }
        _ => panic!("expected conflict"),
    }
}

#[test]
fn parses_two_adjacent_conflicts() {
    let input = "\
<<<<<<< HEAD
a
=======
b
>>>>>>> x
<<<<<<< HEAD
c
=======
d
>>>>>>> x
";
    let regions = parse_markers(input);
    assert_eq!(regions.len(), 2);
    assert!(matches!(regions[0], ParsedRegion::Conflict { .. }));
    assert!(matches!(regions[1], ParsedRegion::Conflict { .. }));
}

#[test]
fn malformed_unpaired_markers_fall_back_to_one_manual_region() {
    let input = "\
<<<<<<< HEAD
ours with no closing marker
line two
";
    let regions = parse_markers(input);
    assert_eq!(regions.len(), 1);
    match &regions[0] {
        ParsedRegion::Conflict { ours, theirs, base } => {
            // Fallback: whole content becomes the ours side, empty theirs.
            assert!(ours.iter().any(|l| l.contains("no closing marker")));
            assert_eq!(theirs, &Vec::<String>::new());
            assert_eq!(base, &None);
        }
        _ => panic!("expected fallback conflict"),
    }
}

#[test]
fn nested_marker_in_theirs_falls_back_to_one_manual_region() {
    let input = "\
<<<<<<< HEAD
ours1
=======
theirs1
<<<<<<< HEAD
ours2
=======
theirs2
>>>>>>> branch
";
    let regions = parse_markers(input);
    assert_eq!(regions.len(), 1);
    match &regions[0] {
        ParsedRegion::Conflict { ours, theirs, base } => {
            // Fallback: whole content becomes the ours side, empty theirs.
            assert!(ours.iter().any(|l| l.contains("theirs1")));
            assert!(ours.iter().any(|l| l.contains("theirs2")));
            assert_eq!(theirs, &Vec::<String>::new());
            assert_eq!(base, &None);
        }
        _ => panic!("expected fallback conflict"),
    }
}

#[test]
fn nested_marker_in_base_falls_back_to_one_manual_region() {
    let input = "\
<<<<<<< HEAD
ours
||||||| base
original
<<<<<<< HEAD
stray nested start
=======
theirs
>>>>>>> branch
";
    let regions = parse_markers(input);
    assert_eq!(regions.len(), 1);
    match &regions[0] {
        ParsedRegion::Conflict { ours, theirs, base } => {
            assert!(ours.iter().any(|l| l.contains("original")));
            assert!(ours.iter().any(|l| l.contains("stray nested start")));
            assert_eq!(theirs, &Vec::<String>::new());
            assert_eq!(base, &None);
        }
        _ => panic!("expected fallback conflict"),
    }
}

#[test]
fn empty_ours_and_theirs_sections_parse_correctly() {
    let input = "\
<<<<<<< HEAD
=======
>>>>>>> branch
";
    let regions = parse_markers(input);
    assert_eq!(regions.len(), 1);
    match &regions[0] {
        ParsedRegion::Conflict { ours, theirs, base } => {
            assert_eq!(ours, &Vec::<String>::new());
            assert_eq!(theirs, &Vec::<String>::new());
            assert_eq!(base, &None);
        }
        _ => panic!("expected conflict"),
    }
}

#[test]
fn malformed_diff3_missing_sep_after_base_falls_back() {
    let input = "\
<<<<<<< HEAD
ours
||||||| base
original with no following separator
";
    let regions = parse_markers(input);
    assert_eq!(regions.len(), 1);
    match &regions[0] {
        ParsedRegion::Conflict { ours, theirs, base } => {
            assert!(ours.iter().any(|l| l.contains("original with no following separator")));
            assert_eq!(theirs, &Vec::<String>::new());
            assert_eq!(base, &None);
        }
        _ => panic!("expected fallback conflict"),
    }
}
