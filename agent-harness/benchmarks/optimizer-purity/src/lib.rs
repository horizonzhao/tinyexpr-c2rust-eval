#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Constant(f64),
    Call { value: f64, pure: bool },
    Add(Box<Expr>, Box<Expr>),
}

pub fn optimize(expr: &mut Expr) {
    match expr {
        Expr::Call { value, .. } => *expr = Expr::Constant(*value),
        Expr::Add(left, right) => {
            optimize(left);
            optimize(right);
            if let (Expr::Constant(a), Expr::Constant(b)) = (&**left, &**right) {
                *expr = Expr::Constant(a + b);
            }
        }
        Expr::Constant(_) => {}
    }
}
