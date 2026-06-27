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
    Qux::process.hook(|next, _this, _value| {
        // first wrapper: call the rest of the chain
        next(_this, _value);
    });
    Qux::process.hook(|next, _this, _value| {
        // second wrapper: call the rest of the chain
        next(_this, _value);
    });
}

// needed for trybuild
fn main() {
    register_wrappers();
    let _ = Qux::init_detours();
}
