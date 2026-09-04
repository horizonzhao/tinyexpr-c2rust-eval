use repair_fn_pointer::{add, same_operation, sub};

#[test]
fn recognizes_matching_operations() {
    assert!(same_operation(add, add));
    assert!(!same_operation(add, sub));
}
