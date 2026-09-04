#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Constant(f64),
    Variable(String),
    Add(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Call { name: String, args: Vec<Expr> },
}

pub fn print_expr(_expr: &Expr) -> String {
    "<expr>".to_string()
}
