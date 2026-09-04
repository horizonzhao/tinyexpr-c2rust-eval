use repair_optimizer_purity::{optimize, Expr};

#[test]
fn folds_pure_calls_and_arithmetic() {
    let mut expr = Expr::Add(
        Box::new(Expr::Call {
            value: 2.0,
            pure: true,
        }),
        Box::new(Expr::Constant(3.0)),
    );
    optimize(&mut expr);
    assert_eq!(expr, Expr::Constant(5.0));
}

#[test]
fn preserves_impure_calls() {
    let original = Expr::Call {
        value: 2.0,
        pure: false,
    };
    let mut expr = original.clone();
    optimize(&mut expr);
    assert_eq!(expr, original);
}
