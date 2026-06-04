use retour_utils::hook_impl;

struct Qux;

#[hook_impl("qux.dll")]
impl Qux {
    #[hook(extern "C" Qux_process, symbol = "process", chain)]
    fn process(this: *mut Self, value: i32) {
        unsafe { Qux_process.call(this, value) }
    }
}

fn register_wrappers() {
    Qux_process__chain.hook(|_this, _value| {
        // first wrapper
    });
    Qux_process__chain.hook(|_this, _value| {
        // second wrapper
    });
}

// needed for trybuild
fn main() {
    register_wrappers();
    unsafe { Qux::init_detours().unwrap() };
}
