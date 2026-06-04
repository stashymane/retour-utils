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
    Lua_close__chain.hook(|state| {
        let _ = state;
    });
    Lua_close__chain.hook(|state| {
        let _ = state;
    });
}

fn main() {
    apply_plugin();
    Lua::init_detours().unwrap();
    // Calling the chain invokes both wrappers then the original:
    // Lua_close__chain.call(std::ptr::null_mut());
}
