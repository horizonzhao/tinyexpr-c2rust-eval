pub type BinaryOp = fn(f64, f64) -> f64;

pub fn add(left: f64, right: f64) -> f64 {
    left + right
}

pub fn sub(left: f64, right: f64) -> f64 {
    left - right
}

pub fn same_operation(left: BinaryOp, right: BinaryOp) -> bool {
    left == right
}
