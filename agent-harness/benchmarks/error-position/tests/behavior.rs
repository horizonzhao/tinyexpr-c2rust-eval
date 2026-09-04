use repair_error_position::reported_error_position;

#[test]
fn parser_errors_keep_the_one_based_offset() {
    assert_eq!(reported_error_position(2, false), 2);
    assert_eq!(reported_error_position(17, false), 17);
}

#[test]
fn allocation_failure_keeps_the_sentinel() {
    assert_eq!(reported_error_position(9, true), -1);
}
