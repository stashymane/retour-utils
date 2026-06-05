use retour_utils::hook_impl;

struct Foo;

#[hook_impl("foo.dll")]
impl Foo {
    #[ptr(offset = 0x1234)]
    pub const some_float: f32 = 0.0;

    #[ptr(offset = 0x5678)]
    pub const some_int: i32 = 0;
}

fn main() {
    let _ = Foo::init_detours();
    // Accessor methods are available
    let _ptr: *mut f32 = some_float.as_ptr();
    let _ptr2: *mut i32 = some_int.as_ptr();
}
