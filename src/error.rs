use crate::{Version, VersionReq};

/// Rscript public error
#[derive(Debug)]
pub enum Error {
    /// Input/Output error,
    Io(std::io::Error),
    /// Bincode error
    Bincode(bincode::Error),
    /// This error is raised if the user attempts to trigger manually a hook on a script and the script is not listening for the specified hook
    ScriptIsNotListeningForHook,
    /// The script is written for a different version of the program
    ScriptVersionMismatch {
        /// The program actual version
        program_actual_version: Version,
        /// The version required of the program by the script
        program_required_version: VersionReq,
    },
    /// Failed to load a dynamic libaray
    DynamicLibError(libloading::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(error) => std::fmt::Display::fmt(error, f),
            Error::Bincode(error) => std::fmt::Display::fmt(error, f),
            Error::ScriptIsNotListeningForHook => write!(
                f,
                "Could not trigger the hook, because the script is not listening for it"
            ),
            Error::ScriptVersionMismatch {
                program_actual_version: program_version,
                program_required_version: script_version,
            } => {
                write!(
                    f,
                    "The scripts requires version: {}, but the program have version: {}",
                    program_version, script_version
                )
            }
            Error::DynamicLibError(error) => {
                write!(f, "Failed to load dynamic library:\n{}", error)
            }
        }
    }
}
impl std::error::Error for Error {}

// Implement From for convenience errors propagation via ?
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<bincode::Error> for Error {
    fn from(error: bincode::Error) -> Self {
        Self::Bincode(error)
    }
}
impl From<libloading::Error> for Error {
    fn from(error: libloading::Error) -> Self {
        Self::DynamicLibError(error)
    }
}
