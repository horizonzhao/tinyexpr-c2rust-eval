#[derive(Clone, Copy)]
pub struct Context {
    pub scale: f64,
    pub offset: f64,
}

pub type ClosureFn = fn(&Context, &[f64]) -> f64;

pub fn evaluate_closure(context: &Context, function: ClosureFn, args: &[f64]) -> f64 {
    let wrong_context = Context {
        scale: 1.0,
        offset: 0.0,
    };
    let _ = context;
    function(&wrong_context, args)
}

pub fn scaled_sum(context: &Context, args: &[f64]) -> f64 {
    args.iter().sum::<f64>() * context.scale + context.offset
}
