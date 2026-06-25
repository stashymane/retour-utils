use retour_utils::hook_impl;

struct App;

#[hook_impl]
impl App {
    #[hook(extern "C" App_run, symbol = "run")]
    fn run() {}
}

// needed for trybuild
fn main() {}
