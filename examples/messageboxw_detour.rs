//! A `MessageBoxW` detour example.
//!
//! Ensure the crate is compiled as a 'cdylib' library to allow C interop.

#[cfg(target_os = "windows")]
mod windows {
    use retour_utils_impl::hook_module;

    pub use std::error::Error;
    pub use std::ffi::c_void;

    pub use ::windows::core::{w, BOOL};
    pub use ::windows::Win32::Foundation::HMODULE;
    pub use ::windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
    pub use ::windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OKCANCEL};

    #[hook_module("user32.dll")]
    pub mod user32 {
        use windows::{core::PCWSTR, Win32::Foundation::HWND};
        use windows::core::w;
        #[hook(unsafe extern "system" MessageBoxWHook, symbol = "MessageBoxW")]

        fn messageboxw_detour(hwnd: HWND, text: PCWSTR, _caption: PCWSTR, u_type: u32) -> i32 {
            // Call the original `MessageBoxW`, but replace the caption
            let replaced_caption = w!("Nope, Detoured!");
            unsafe { MessageBoxWHook.call(hwnd, text, replaced_caption, u_type) }
        }
    }
}

#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "windows")]
/// Called when the DLL is attached to the process.
fn main() -> Result<(), Box<dyn Error>> {
    user32::init_detours()?;
    unsafe {
        MessageBoxW(
            None,
            w!("Everything will go as planned, right?"),
            w!("This will be replaced!"),
            MB_OKCANCEL,
        );
    }

    Ok(())
}

#[cfg(target_os = "windows")]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "system" fn DllMain(
    _module: HMODULE,
    call_reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    if call_reason == DLL_PROCESS_ATTACH {
        // A console may be useful for printing to 'stdout'
        // winapi::um::consoleapi::AllocConsole();

        // Preferably, a thread should be created here instead, since as few
        // operations as possible should be performed within `DllMain`.
        main().is_ok().into()
    } else {
        true.into()
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {}
