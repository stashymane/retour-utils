use retour_utils::hook_module;
use std::sync::Mutex;

static ORDER: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
const ORIGINAL_NAME: &str = "original";

pub extern "C" fn real_fn(value: i32) -> i32 {
    ORDER.lock().unwrap().push(ORIGINAL_NAME);
    value * 2
}

#[hook_module]
mod hooks {
    #[hook(pub extern "C" OriginalFnHook, offset = 0x0, chain)]
    pub fn original_fn(value: i32) -> i32 {
        OriginalFnHook.call(value)
    }
}

#[test]
fn chain_hooks_run_in_order_module() {
    let first = "first";
    let second = "second";

    hooks::original_fn.hook(|value, next| {
        ORDER.lock().unwrap().push(first);
        next(value)
    });
    hooks::original_fn.hook(|value, next| {
        ORDER.lock().unwrap().push(second);
        next(value)
    });

    // Initialize the detour directly using the known function pointer address.
    unsafe {
        hooks::OriginalFnHook
            .initialize(
                retour::Function::from_ptr(real_fn as *const ()),
                hooks::__original_fn_detour,
            )
            .unwrap()
            .enable()
            .unwrap();
    }

    let result = hooks::original_fn.call(21);

    let order = ORDER.lock().unwrap();
    assert_eq!(
        *order,
        vec![second, first, ORIGINAL_NAME],
        "hooks must run most-recent-first, then original last"
    );
    assert_eq!(result, 42);
}
