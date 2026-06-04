use retour_utils::hook_impl;

struct Baz;

#[hook_impl("baz.dll")]
impl Baz {
    #[hook(extern "C" Baz_func, offset = 0x1234)]
    fn func(x: f32) -> f32 {
        unsafe { Baz_func.call(x) }
    }
}

// needed for trybuild
fn main() {
    Baz::init_detours().unwrap();
}
