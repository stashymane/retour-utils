use retour_utils::hook_impl;

struct Qux;

#[hook_impl("qux.dll")]
impl Qux {
    #[hook(extern "C" Qux_process, symbol = "process", chain)]
    fn process(this: *mut Self, value: i32) {
        Qux_process.call(this, value)
    }
}

fn register_wrappers() {
    Qux::process.hook(|_this, _value, next| {
        // first wrapper: call the rest of the chain
        next(_this, _value);
    });
    Qux::process.hook(|_this, _value, next| {
        // second wrapper: call the rest of the chain
        next(_this, _value);
    });
}

// needed for trybuild
fn main() {
    register_wrappers();
    Qux::init_detours().unwrap();
}
