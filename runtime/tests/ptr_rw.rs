use retour_utils::hook_impl;

struct Foo;

#[hook_impl("foo.dll")]
impl Foo {
    #[ptr(offset = 0x1234)]
    pub const SOME_FLOAT: f32 = 0.0;

    #[ptr(offset = 0x5678)]
    pub const SOME_INT: i32 = 0;
}

#[test]
fn ptr_read_write_float() {
    let mut value: f32 = 1.0;
    // Point the accessor at our local variable instead of calling init_detours
    // (which would try to load "foo.dll").
    Foo::SOME_FLOAT
        .addr
        .store(&raw mut value as usize, ::std::sync::atomic::Ordering::Relaxed);

    // SAFETY: `value` is alive and the address was just stored above.
    unsafe {
        assert_eq!(Foo::SOME_FLOAT.read(), 1.0_f32);

        Foo::SOME_FLOAT.write(42.0);
        assert_eq!(value, 42.0_f32);
        assert_eq!(Foo::SOME_FLOAT.read(), 42.0_f32);
    }
}

#[test]
fn ptr_read_write_int() {
    let mut value: i32 = 100;
    Foo::SOME_INT
        .addr
        .store(&raw mut value as usize, ::std::sync::atomic::Ordering::Relaxed);

    unsafe {
        assert_eq!(Foo::SOME_INT.read(), 100_i32);

        Foo::SOME_INT.write(-7);
        assert_eq!(value, -7_i32);
        assert_eq!(Foo::SOME_INT.read(), -7_i32);
    }
}
