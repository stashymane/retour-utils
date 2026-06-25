use retour_utils::hook_module;

#[hook_module("foo.dll")]
mod foo {
    #[ptr(offset = 0x1234)]
    pub const some_float: f32 = 0.0;

    #[ptr(offset = 0x5678)]
    pub const some_int: i32 = 0;
}

fn main() {
    let _ = foo::init_detours();
    // Accessor methods are available
    let _ptr: *mut f32 = foo::some_float.as_ptr();
    let _ptr2: *mut i32 = foo::some_int.as_ptr();
}
