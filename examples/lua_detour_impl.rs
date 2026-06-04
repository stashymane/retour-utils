use retour_utils::hook_impl;

struct Lua;

#[allow(non_camel_case_types)]
type lua_State = ();
#[allow(non_camel_case_types)]
type lua_Alloc = ();

#[hook_impl("lua52.dll")]
impl Lua {
    #[hook(unsafe extern "C" Lua_newstate, symbol = "Lua_newstate")]
    pub fn newstate(f: *mut lua_Alloc, ud: *mut std::ffi::c_void) -> *mut lua_State {
        unsafe { Lua_newstate.call(f, ud) }
    }

    #[hook(unsafe extern "C" Lua_close, symbol = "Lua_close", chain)]
    pub fn close(state: *mut Self) {
        unsafe { Lua_close.call(state) }
    }
}

fn apply_plugin() {
    Lua::close.hook(|state, next| {
        // first wrapper: do something, then call the rest of the chain
        next(state);
    });
    Lua::close.hook(|state, next| {
        // second wrapper: do something, then call the rest of the chain
        next(state);
    });
}

fn main() {
    apply_plugin();
    Lua::init_detours().unwrap();
    // Calling the chain invokes both wrappers then the original:
    // Lua::close.call(std::ptr::null_mut());
}
