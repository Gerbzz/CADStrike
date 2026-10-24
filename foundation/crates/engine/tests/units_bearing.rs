use cadcraft_engine::units::{AngleSettings, parse_angle};

#[test]
fn bearing_rejects_out_of_range_angles() {
    for s in ["n100e", "n-45e", "n91e", "s-1w", "n-0.5e", "s91e", "n91w", "s91w", "N90d0'1\"E", "S-0d0'1\"W"] {
        assert_eq!(parse_angle(s), None, "{s}");
    }
}

#[test]
fn bearing_keeps_quadrants_and_inclusive_boundaries() {
    for (s, degrees) in [
        ("n0e", 90.0),
        ("n90e", 0.0),
        ("n0w", 90.0),
        ("n90w", 180.0),
        ("s0w", 270.0),
        ("s90w", 180.0),
        ("s0e", 270.0),
        ("s90e", 360.0),
        (" N 45d30' E ", 44.5),
        ("N45dW", 135.0),
        ("s45w", 225.0),
        ("s45e", 315.0),
    ] {
        let actual = parse_angle(s).unwrap().to_degrees();
        assert!((actual - degrees).abs() < 1e-9, "{s}: {actual} != {degrees}");
    }
}

#[test]
fn bearing_stays_absolute_and_does_not_restrict_other_angle_units() {
    let settings = AngleSettings { aunits: 3, angbase: 0.5, clockwise: true };
    assert!((settings.direction("n45e").unwrap().to_degrees() - 45.0).abs() < 1e-9);
    assert_eq!(settings.direction("n91e"), None);
    for (s, degrees) in [("-45", -45.0), ("100d", 100.0), ("200g", 180.0)] {
        assert!((parse_angle(s).unwrap().to_degrees() - degrees).abs() < 1e-9, "{s}");
    }
    assert_eq!(parse_angle("-1r"), Some(-1.0));
    for s in ["nNaNe", "ninfw", "s-infe", "n1e309e"] {
        assert_eq!(parse_angle(s), None, "{s}");
    }
}
