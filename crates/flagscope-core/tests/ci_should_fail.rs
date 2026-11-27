//! Deliberately failing test for CA-14.2; never merged.

#[test]
fn deliberately_failing_test() {
    assert_eq!(1, 2, "this test must fail in CI");
}
