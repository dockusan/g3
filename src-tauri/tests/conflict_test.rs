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
