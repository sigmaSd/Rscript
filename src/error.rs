/// Rscript public error
#[derive(Debug)]
pub enum Error {
    /// Bincode error
    Bincode(bincode::Error),
    /// This error is raised if the user attempts to trigger manually a hook an a script and the script is not listening for the specified hook
    ScriptIsNotListeningForHook,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Bincode(error) => write!(f, "{}", error),
            Error::ScriptIsNotListeningForHook => write!(
                f,
                "Could not trigger the hook, because the script is not listening for specified hook"
            ),
        }
    }
}
impl std::error::Error for Error {}
