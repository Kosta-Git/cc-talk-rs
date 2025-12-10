#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-parses.rs");
    t.pass("tests/02-simple-poll.rs");

    t.compile_fail("tests/no_compile/enum-fails.rs");
    t.compile_fail("tests/no_compile/union-fails.rs");
}
