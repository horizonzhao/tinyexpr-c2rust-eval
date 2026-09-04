use repair_power_associativity::eval_power_chain;

#[test]
fn matches_tinyexpr_left_associativity() {
    assert_eq!(eval_power_chain(&[2.0, 3.0, 2.0]), Some(64.0));
}

#[test]
fn handles_short_chains() {
    assert_eq!(eval_power_chain(&[5.0]), Some(5.0));
    assert_eq!(eval_power_chain(&[]), None);
}
