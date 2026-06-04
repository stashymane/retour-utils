use retour_utils::hook_impl;

struct Lua;

#[hook_impl("lua52.dll")]
impl Lua {
    #[hook(unsafe extern "C" Lua_close, symbol = "Lua_close")]
    pub fn close(state: *mut Self) {
        unsafe { Lua_close.call(state) }
    }
}

fn main() {
    unsafe { Lua::initialize_hooks().unwrap() };
}
