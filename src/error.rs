use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    /// Detour error encountered within the [`retour`] crate
    DetourError(retour::Error),
    /// Module trying to be hooked could not be loaded
    ModuleNotLoaded,
    /// Module loaded but the requested symbol or offset could not be resolved
    SymbolNotFound,
    /// OS-level error when loading a module
    IoError(std::io::Error),
}

impl From<retour::Error> for Error {
    fn from(value: retour::Error) -> Self {
        Error::DetourError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DetourError(e) => f.write_fmt(format_args!("Detour Error: {e:?}")),
            Error::ModuleNotLoaded => f.write_str("Module could not be loaded"),
            Error::SymbolNotFound => f.write_str("Symbol or offset could not be resolved in the module"),
            Error::IoError(e) => f.write_fmt(format_args!("IO error loading module: {e}")),
        }
    }
}
impl std::error::Error for Error {}
