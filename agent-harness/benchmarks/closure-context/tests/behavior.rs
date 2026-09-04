use repair_closure_context::{evaluate_closure, scaled_sum, Context};

#[test]
fn passes_the_registered_context_to_the_closure() {
    let context = Context {
        scale: 3.0,
        offset: 2.0,
    };
    assert_eq!(evaluate_closure(&context, scaled_sum, &[1.0, 4.0]), 17.0);
}
