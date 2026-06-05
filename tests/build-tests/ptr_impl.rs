use retour_utils::hook_impl;

struct Foo;

#[hook_impl("foo.dll")]
impl Foo {
    #[ptr(offset = 0x1234)]
    pub const SOME_FLOAT: f32 = 0.0;

    #[ptr(offset = 0x5678)]
    pub const SOME_INT: i32 = 0;
}

fn main() {
    let _ = Foo::init_detours();
    // Accessor methods are available
    let _ptr: *mut f32 = Foo::SOME_FLOAT.as_ptr();
    let _ptr2: *mut i32 = Foo::SOME_INT.as_ptr();
}
