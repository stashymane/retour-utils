use retour_utils::hook_impl;
use std::sync::Mutex;

static ORDER: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
const ORIGINAL_NAME: &str = "original";

pub extern "C" fn original_fn(value: i32) -> i32 {
    ORDER.lock().unwrap().push(ORIGINAL_NAME);
    value * 2
}

struct Hooks;

#[hook_impl]
impl Hooks {
    #[hook(extern "C" OriginalFnHook, offset = 0x0, chain)]
    fn original_fn(value: i32) -> i32 {
        OriginalFnHook.call(value)
    }
}

#[test]
fn chain_hooks_run_in_order() {
    let first = "first";
    let second = "second";

    Hooks::original_fn.hook(|value, next| {
        ORDER.lock().unwrap().push(first);
        next(value)
    });
    Hooks::original_fn.hook(|value, next| {
        ORDER.lock().unwrap().push(second);
        next(value)
    });

    // Initialize the detour directly using the known function pointer address.
    unsafe {
        OriginalFnHook
            .initialize(
                retour::Function::from_ptr(original_fn as *const ()),
                Hooks::__original_fn_detour,
            )
            .unwrap()
            .enable()
            .unwrap();
    }

    let result = Hooks::original_fn.call(21);

    let order = ORDER.lock().unwrap();
    assert_eq!(
        *order,
        vec![second, first, ORIGINAL_NAME],
        "hooks must run most-recent-first, then original last"
    );
    assert_eq!(result, 42);
}
