#![warn(missing_docs)]

//! Crate to easily script any rust project
//! # Rscript
//! The main idea is:
//! - Create a new crate (my-project-api for example)
//! - Add hooks to this api-crate
//! - This api-crate should be used by the main-crate and by the scripts
//! - Trigger Hooks in the main crate
//! - Receive the hooks on the script side, and react to them with any output
//!
//!
//! Goals:
//! - Be as easy as possible to include on already established projects
//! - Strive for maximum compile time guarantees
//!
//! This crate was extracted from [IRust](https://github.com/sigmaSd/IRust)
//!
//! Taking *IRust* as an example:
//! - It has an API crate where hooks are defined [irust_api](https://github.com/sigmaSd/IRust/blob/master/crates/irust_api/src/lib.rs#L22)
//! - It trigger hooks on the main crate [irust](https://github.com/sigmaSd/IRust/blob/master/crates/irust/src/irust.rs#L136)
//! - And a script example [vim_mode](https://github.com/sigmaSd/IRust/tree/master/scripts_examples/script4/irust_vim)
//!
//! Check out the [examples](https://github.com/sigmaSd/Rscript/tree/master/examples) for more info.

use scripting::{FFiData, FFiStr};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    env,
    path::Path,
    process::{Child, Stdio},
};

// Rexport Version, VersionReq
/// *SemVer version* as defined by <https://semver.org.>\
/// The main crate must specify its version when adding scripts to [ScriptManager]
pub use semver::Version;
/// *SemVer version requirement* describing the intersection of some version comparators, such as >=1.2.3, <1.8.\
/// Each script must specify the required version of the main crate when responding to [Message::Greeting]
pub use semver::VersionReq;

pub mod scripting;

mod error;
pub use error::Error;

use crate::scripting::DynamicScript;

/// Script metadata that every script should send to the main_crate  when starting up after receiving the greeting message [Message::Greeting]
#[derive(Serialize, Deserialize, Debug)]
pub struct ScriptInfo {
    /// Script name
    pub name: String,
    /// Script type: Daemon/OneShot
    pub script_type: ScriptType,
    /// The hooks that the script wants to listen to
    pub hooks: Box<[String]>,
    /// The version requirement of the program that the script will run against
    pub version_requirement: VersionReq,
}

impl ScriptInfo {
    /// Create a new script metadata, the new constructor tries to add more ergonomics
    pub fn new(
        name: &'static str,
        script_type: ScriptType,
        hooks: &'static [&'static str],
        version_requirement: VersionReq,
    ) -> Self {
        Self {
            name: name.into(),
            script_type,
            hooks: hooks.iter().map(|hook| String::from(*hook)).collect(),
            version_requirement,
        }
    }
    /// Serialize `ScriptInfo` into `FFiData`
    /// This is needed for writing [ScriptType::DynamicLib] scripts
    pub fn into_ffi_data(self) -> FFiData {
        FFiData::serialize_from(&self).expect("ScriptInfo is always serialize-able")
    }
}

/// ScriptType: Daemon/OneShot/DynamicLib
/// - *OneShot* scripts are expected to be spawned(process::Command::new) by the main crate ach time they are used, this should be preferred if performance and keeping state are not a concern since it has some nice advantage which is the allure of hot reloading (recompiling the script will affect the main crate while its running)
///
/// - *Daemon* scripts are expected to run indefinitely, the main advantage is better performance and keeping the state
///
/// - *DynamicLib* scripts compiled as dynamic libraries, the main advantage is even better performance, but this is the least safe option
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ScriptType {
    /// Scripts that is executed each time
    OneShot,
    /// Scripts that runs indefinitely, it will continue to receive and send hooks while its
    /// running
    Daemon,
    /// Script compiled as a dynamic library\
    /// It needs to export a static [DynamicScript] instance with [DynamicScript::NAME] as name (with `#[no_mangle]` attribute)
    DynamicLib,
}

/// ScriptManager holds all the scripts found, it can be constructed with [ScriptManager::default]\
/// Initially its empty, to populate it, we can use one of the methods to add scripts, currently only [ScriptManager::add_scripts_by_path] is provided
#[derive(Default)]
pub struct ScriptManager {
    scripts: Vec<Script>,
}

/// Message that is sent from the main crate to the script each time it wants to interact with it\
/// Greeting message must be sent when looking for scripts\
/// Execute message must be sent each time a hook is triggered
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Message {
    /// Greet a script, the script must respond with [ScriptInfo]
    Greeting,
    /// Must be sent each time a hook is triggered
    Execute,
}

impl ScriptManager {
    /// Look for scripts in the specified folder\
    /// It requires specifying a [VersionReq] so the script manager can check for incompatibility and if that's the case it will return an error: [Error::ScriptVersionMismatch]\
    /// The script manager will send a [Message::Greeting] for every script found and the scripts must respond with [ScriptInfo]
    ///
    /// ```rust, no_run
    /// # use rscript::*;
    /// let mut sm = ScriptManager::default();
    /// let scripts_path: std::path::PathBuf = todo!(); // Defined by the user
    /// const VERSION: &'static str = concat!("main_crate-", env!("CARGO_PKG_VERSION"));
    /// sm.add_scripts_by_path(scripts_path, Version::parse(VERSION).expect("version is correct"));
    /// ```
    pub fn add_scripts_by_path<P: AsRef<Path>>(
        &mut self,
        path: P,
        version: Version,
    ) -> Result<(), Error> {
        fn start_script(path: &Path, version: &Version) -> Result<Script, Error> {
            let mut script = std::process::Command::new(path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

            // Send Greeting Message
            let stdin = script.stdin.as_mut().expect("stdin is piped");
            bincode::serialize_into(stdin, &Message::Greeting)?;

            // Receive ScriptInfo
            let stdout = script.stdout.as_mut().expect("stdout is piped");
            let metadata: ScriptInfo = bincode::deserialize_from(stdout)?;

            // Check if the provided version matches the script version
            if !metadata.version_requirement.matches(version) {
                return Err(Error::ScriptVersionMismatch {
                    program_actual_version: version.clone(),
                    program_required_version: metadata.version_requirement,
                });
            }

            // Save script depending on its type
            let script = if matches!(metadata.script_type, ScriptType::Daemon) {
                ScriptTypeInternal::Daemon(script)
            } else {
                ScriptTypeInternal::OneShot(path.to_path_buf())
            };
            Ok(Script {
                script,
                metadata,
                state: State::Active,
            })
        }
        let path = path.as_ref();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == env::consts::DLL_EXTENSION {
                        continue;
                    }
                }
                self.scripts.push(start_script(&path, &version)?);
            }
        }
        Ok(())
    }
    /// Same as [ScriptManager::add_scripts_by_path] but looks for dynamic libraries instead
    ///
    /// # Safety
    /// See <https://docs.rs/libloading/0.7.1/libloading/struct.Library.html#safety>
    pub unsafe fn add_dynamic_scripts_by_path<P: AsRef<Path>>(
        &mut self,
        path: P,
        version: Version,
    ) -> Result<(), Error> {
        fn load_dynamic_library(path: &Path, version: &Version) -> Result<Script, Error> {
            let lib = unsafe { libloading::Library::new(path)? };
            let script: libloading::Symbol<&DynamicScript> =
                unsafe { lib.get(DynamicScript::NAME)? };

            let metadata: ScriptInfo = (script.script_info)().deserialize()?;
            if !metadata.version_requirement.matches(version) {
                return Err(Error::ScriptVersionMismatch {
                    program_actual_version: version.clone(),
                    program_required_version: metadata.version_requirement,
                });
            }
            Ok(Script {
                script: ScriptTypeInternal::DynamicLib(lib),
                metadata,
                state: State::Active,
            })
        }
        let path = path.as_ref();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == env::consts::DLL_EXTENSION {
                        self.scripts.push(load_dynamic_library(&path, &version)?);
                    }
                }
            }
        }
        Ok(())
    }
    /// Trigger a hook
    /// All scripts that are *active* and that are listening for this particular hook will receive it
    pub fn trigger<'a, H: 'static + Hook>(
        &'a mut self,
        hook: H,
    ) -> impl Iterator<Item = Result<<H as Hook>::Output, Error>> + 'a {
        self.scripts.iter_mut().filter_map(move |script| {
            if script.is_active() && script.is_listening_for::<H>() {
                Some(script.trigger_internal(&hook))
            } else {
                None
            }
        })
    }
    /// List of current scripts
    pub fn scripts(&self) -> &[Script] {
        &self.scripts
    }
    /// Mutable list of current scripts, useful for activating/deactivating a script
    pub fn scripts_mut(&mut self) -> &mut [Script] {
        &mut self.scripts
    }
}

impl Drop for ScriptManager {
    fn drop(&mut self) {
        self.scripts.iter_mut().for_each(|script| script.end());
    }
}

/// A script abstraction
// The user should not be able to construct a Script manually
#[derive(Debug)]
pub struct Script {
    metadata: ScriptInfo,
    script: ScriptTypeInternal,
    state: State,
}

#[derive(Debug)]
enum State {
    Active,
    Inactive,
}

#[derive(Debug)]
enum ScriptTypeInternal {
    Daemon(Child),
    OneShot(std::path::PathBuf),
    DynamicLib(libloading::Library),
}

impl Script {
    //public
    /// Returns the script metadata
    pub fn metadata(&self) -> &ScriptInfo {
        &self.metadata
    }
    /// Activate a script, inactive scripts will not react to hooks
    pub fn activate(&mut self) {
        self.state = State::Active;
    }
    /// Deactivate a script, inactive scripts will not react to hooks
    pub fn deactivate(&mut self) {
        self.state = State::Inactive;
    }
    /// Query the script state
    pub fn is_active(&self) -> bool {
        matches!(self.state, State::Active)
    }
    /// Check if a script is listening for a hook
    pub fn is_listening_for<H: Hook>(&self) -> bool {
        self.metadata
            .hooks
            .iter()
            .any(|hook| hook.as_str() == H::NAME)
    }
    /// Trigger a hook on the script, this disregards the script state as in the hook will be triggered even if the script is inactive\
    /// If the script is not listening for the specified hook, an error will be returned
    pub fn trigger<H: Hook>(&mut self, hook: &H) -> Result<<H as Hook>::Output, Error> {
        if self.is_listening_for::<H>() {
            self.trigger_internal(hook)
        } else {
            Err(Error::ScriptIsNotListeningForHook)
        }
    }
}

impl Script {
    // private
    fn trigger_internal<H: Hook>(&mut self, hook: &H) -> Result<<H as Hook>::Output, Error> {
        let trigger_hook_common =
            |script: &mut Child| -> Result<<H as Hook>::Output, bincode::Error> {
                let mut stdin = script.stdin.as_mut().expect("stdin is piped");
                let stdout = script.stdout.as_mut().expect("stdout is piped");

                // Send Execute message
                bincode::serialize_into(&mut stdin, &Message::Execute)?;
                // bincode write hook type
                bincode::serialize_into(&mut stdin, H::NAME)?;
                // bincode write hook
                bincode::serialize_into(stdin, hook)?;
                // bincode read result -> O
                bincode::deserialize_from(stdout)
            };

        Ok(match &mut self.script {
            ScriptTypeInternal::Daemon(ref mut script) => trigger_hook_common(script)?,
            ScriptTypeInternal::OneShot(script_path) => trigger_hook_common(
                &mut std::process::Command::new(script_path)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()?,
            )?,
            ScriptTypeInternal::DynamicLib(lib) => unsafe {
                let script: libloading::Symbol<&DynamicScript> = lib.get(DynamicScript::NAME)?;

                let output = (script.script)(FFiStr::new(H::NAME), FFiData::serialize_from(hook)?);
                output.deserialize()?
            },
        })
    }
    fn end(&mut self) {
        // This errors if the script has already exited
        // We don't care about this error
        if let ScriptTypeInternal::Daemon(ref mut script) = self.script {
            let _ = script.kill();
        }
    }
}

/// Trait to mark the hooks that will be triggered in the main crate\
/// Triggering the hook sends input to the script, and receive the output from it\
/// The output type is declared on the hook associated type\
/// The associated NAME is needed in order to differentiate the hooks received in the script\
/// The hook struct is required to implement serde::Serialize+Deserialize, so it can be used by bincode\
/// The hooks should be declared on an external crate (my-project-api for example) so they can be used both by the main crate and the script\
/// ```rust
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct Eval(String);
/// impl rscript::Hook for Eval {
///     const NAME: &'static str = "Eval";
///     type Output = Option<String>;
/// }
pub trait Hook: Serialize + DeserializeOwned {
    /// The name of the hook, required to distinguish the received hook on the script side
    const NAME: &'static str;
    /// The output type of the script
    type Output: Serialize + DeserializeOwned;
}
