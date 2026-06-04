use retour_utils::hook_impl;

struct Bar;

#[hook_impl("bar.dll")]
impl Bar {
    #[hook(extern "C" Bar_method, symbol = "method")]
    fn method(this: *mut Self, value: i32) -> i32 {
        unsafe { Bar_method.call(this, value) }
    }
}

// needed for trybuild
fn main() {
    Bar::init_detours().unwrap();
}
