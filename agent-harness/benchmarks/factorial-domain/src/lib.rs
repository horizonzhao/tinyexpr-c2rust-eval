pub fn factorial(value: f64) -> f64 {
    let mut result = 1.0;
    let mut current = value as u64;
    while current > 1 {
        result *= current as f64;
        current -= 1;
    }
    result
}
