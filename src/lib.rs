#![deny(missing_docs)]

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

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    path::Path,
    process::{Child, Stdio},
};

/// Module that contains traits that improves writing scripts experience
pub mod scripting;

/// Script metadata that every script should send to the main_crate  when starting up after receiving the greeting message [Message::Greeting]
#[derive(Serialize, Deserialize, Debug)]
pub struct ScriptInfo {
    /// Script name
    pub name: String,
    /// Script type: Daemon/OneShot
    pub script_type: ScriptType,
    /// The hooks that the script wants to listen to
    pub hooks: Box<[String]>,
}
impl ScriptInfo {
    /// Create a new script metadata, the new constructor tries to add more ergonomics
    pub fn new(
        name: &'static str,
        script_type: ScriptType,
        hooks: &'static [&'static str],
    ) -> Self {
        Self {
            name: name.into(),
            script_type,
            hooks: hooks.iter().map(|hook| String::from(*hook)).collect(),
        }
    }
}

/// ScriptType: Daemon/OneShot
/// - *OneShot* scripts are expected to be spawned(process::Command::new) by the main crate each time they are used, this should be preferred if performance and keeping state are not a concern since it has some nice advantage which is the allure of hot reloading (recompiling the script will affect the main crate while its running)
///
/// - *Daemon* scripts are expected to run indefinitely, the main advantage is better performance and keeping the state
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ScriptType {
    /// Scripts that is executed each time
    OneShot,
    /// Scripts that runs indefinitely, it will continue to receive and send hooks while its
    /// running
    Daemon,
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
    /// Look for scripts in the specified folder
    /// The script manager will send a [Message::Greeting] for every script found and the scripts must respond with [ScriptInfo]
    pub fn add_scripts_by_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), bincode::Error> {
        fn start_script(path: &Path) -> Result<Script, bincode::Error> {
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
                self.scripts.push(start_script(&path)?);
            }
        }
        Ok(())
    }
    /// Trigger a hook
    /// All scripts that are *active* and that are listening for this particular hook will receive it
    pub fn trigger<'a, H: 'static + Hook>(
        &'a mut self,
        hook: H,
    ) -> impl Iterator<Item = Result<<H as Hook>::Output, bincode::Error>> + 'a {
        self.scripts.iter_mut().filter_map(move |script| {
            if script.is_active() && script.is_listening_for::<H>() {
                Some(script.trigger(&hook))
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
}
impl Script {
    // private
    fn is_listening_for<H: Hook>(&self) -> bool {
        self.metadata
            .hooks
            .iter()
            .any(|hook| hook.as_str() == H::NAME)
    }
    fn trigger<H: Hook>(&mut self, hook: &H) -> Result<<H as Hook>::Output, bincode::Error> {
        let trigger_hook_common = |script: &mut Child| {
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

        match &mut self.script {
            ScriptTypeInternal::Daemon(ref mut script) => trigger_hook_common(script),
            ScriptTypeInternal::OneShot(script_path) => trigger_hook_common(
                &mut std::process::Command::new(script_path)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()?,
            ),
        }
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
/// example:
/// ```rust
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct Eval(String);
/// impl rscript::Hook for Eval {
///     const NAME: &'static str = "Eval";
///     type Output = Option<String>;
/// }
pub trait Hook: Serialize {
    /// The name of the hook, required to distinguish the received hook on the script side
    const NAME: &'static str;
    /// The output type of the script
    type Output: DeserializeOwned;
}
