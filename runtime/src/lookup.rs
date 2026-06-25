use minidl::Library;
use std::ffi::CString;

pub enum LookupData {
    Offset {
        module: &'static str,
        offset: usize,
    },
    Symbol {
        module: &'static str,
        symbol: &'static str,
    },
    /// Offset relative to the current executable/process base address.
    SelfOffset {
        offset: usize,
    },
    /// Exported symbol from the current executable/process.
    SelfSymbol {
        symbol: &'static str,
    },
}

impl LookupData {
    pub const fn from_offset(module: &'static str, offset: usize) -> Self {
        Self::Offset { module, offset }
    }

    pub const fn from_symbol(module: &'static str, symbol: &'static str) -> Self {
        Self::Symbol { module, symbol }
    }

    pub const fn from_self_offset(offset: usize) -> Self {
        Self::SelfOffset { offset }
    }

    pub const fn from_self_symbol(symbol: &'static str) -> Self {
        Self::SelfSymbol { symbol }
    }

    pub(crate) fn get_module(&self) -> Option<&str> {
        match self {
            Self::Offset { module, .. } => Some(module),
            Self::Symbol { module, .. } => Some(module),
            Self::SelfOffset { .. } | Self::SelfSymbol { .. } => None,
        }
    }

    pub(crate) fn address_from_handle(&self, handle: &Library) -> Option<*const ()> {
        match self {
            LookupData::Offset { offset, .. } | LookupData::SelfOffset { offset } => {
                Some((handle.as_ptr() as usize + offset) as *const ())
            }
            LookupData::Symbol { symbol, .. } | LookupData::SelfSymbol { symbol } => {
                let c_symbol = CString::new(*symbol).ok()?;
                let symbol_with_null_terminator =
                    String::from_utf8(c_symbol.into_bytes_with_nul()).ok()?;

                unsafe { handle.sym_opt(&symbol_with_null_terminator) }
            }
        }
    }
}
