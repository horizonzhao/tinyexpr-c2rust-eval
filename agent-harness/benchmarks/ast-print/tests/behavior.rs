use repair_ast_print::{print_expr, Expr};

#[test]
fn prints_leaves() {
    assert_eq!(print_expr(&Expr::Constant(3.5)), "3.5");
    assert_eq!(print_expr(&Expr::Variable("x".into())), "x");
}

#[test]
fn prints_nested_operations_with_explicit_grouping() {
    let expr = Expr::Multiply(
        Box::new(Expr::Add(
            Box::new(Expr::Variable("x".into())),
            Box::new(Expr::Constant(2.0)),
        )),
        Box::new(Expr::Constant(4.0)),
    );
    assert_eq!(print_expr(&expr), "((x + 2) * 4)");
}

#[test]
fn prints_function_arguments() {
    let expr = Expr::Call {
        name: "pow".into(),
        args: vec![Expr::Variable("x".into()), Expr::Constant(2.0)],
    };
    assert_eq!(print_expr(&expr), "pow(x, 2)");
}
