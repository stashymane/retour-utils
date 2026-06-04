use retour_utils::hook_impl;

struct Foo;

#[hook_impl("foo.dll")]
impl Foo {
    #[hook(extern "C" Foo_bar, symbol = "bar")]
    fn bar(x: i32) -> i32 {
        Foo_bar.call(x)
    }
}

// needed for trybuild
fn main() {
    let _ = Foo::init_detours();
}
