use crate::Version;

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
        /// The version of the program expected by the script
        program_expected_version: Version,
    },
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(error) => write!(f, "{}", error),
            Error::Bincode(error) => write!(f, "{}", error),
            Error::ScriptIsNotListeningForHook => write!(
                f,
                "Could not trigger the hook, because the script is not listening for it"
            ),
            Error::ScriptVersionMismatch {
                program_actual_version: program_version,
                program_expected_version: script_version,
            } => {
                write!(
                    f,
                    "The scripts expects version: {}, but the program have version: {}",
                    program_version, script_version
                )
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
