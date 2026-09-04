use repair_format_g::format_g;

#[test]
fn uses_six_significant_digits() {
    assert_eq!(format_g(std::f64::consts::PI), "3.14159");
    assert_eq!(format_g(1.0 / 3.0), "0.333333");
}

#[test]
fn switches_to_c_style_scientific_notation() {
    assert_eq!(format_g(1e6), "1e+06");
    assert_eq!(format_g(1e-5), "1e-05");
}

#[test]
fn handles_special_values() {
    assert_eq!(format_g(f64::NAN), "nan");
    assert_eq!(format_g(f64::INFINITY), "inf");
    assert_eq!(format_g(f64::NEG_INFINITY), "-inf");
}
