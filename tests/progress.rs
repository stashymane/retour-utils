#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/build-tests/retain_other_items.rs");
    t.pass("tests/build-tests/allow_unnamed_module.rs");
    t.pass("tests/build-tests/maintain_vis.rs")
}

#[test]
fn build_abi_types() {
    let t = trybuild::TestCases::new();
    t.pass("tests/build-tests/different_abis.rs");
}

#[test]
fn hook_impl_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/build-tests/hook_impl_basic.rs");
    t.pass("tests/build-tests/hook_impl_self.rs");
    t.pass("tests/build-tests/hook_impl_offset.rs");
    t.pass("tests/build-tests/hook_impl_chain.rs");
    t.pass("tests/build-tests/hook_impl_no_dll.rs");
}

#[test]
fn ptr_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/build-tests/ptr_module.rs");
    t.pass("tests/build-tests/ptr_impl.rs");
}
