use repair_factorial_domain::factorial;

#[test]
fn computes_nonnegative_integers() {
    assert_eq!(factorial(0.0), 1.0);
    assert_eq!(factorial(1.0), 1.0);
    assert_eq!(factorial(5.0), 120.0);
}

#[test]
fn rejects_values_outside_the_function_domain() {
    assert!(factorial(-1.0).is_nan());
    assert!(factorial(2.5).is_nan());
    assert!(factorial(f64::INFINITY).is_nan());
}
