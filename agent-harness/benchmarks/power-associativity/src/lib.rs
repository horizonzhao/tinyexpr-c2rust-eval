pub fn eval_power_chain(values: &[f64]) -> Option<f64> {
    values
        .iter()
        .rev()
        .copied()
        .reduce(|right, left| left.powf(right))
}
