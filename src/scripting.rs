use crate::{Hook, VersionReq};

use super::{Message, ScriptInfo, ScriptType};
use std::io::Write;

/// Trait that can be implemented on a script abstraction struct\
/// The implementer should provide [Scripter::script_type], [Scripter::name], [Scripter::hooks] and [Scripter::version_requirement]\
///  The struct should call [Scripter::execute]\
///  ```rust, no_run
///  # use rscript::*;
///  # use rscript::scripting::Scripter;
///
///  // The hook should usually be on a common api crate.
///  #[derive(serde::Serialize, serde::Deserialize)]
///  struct MyHook;
///  impl Hook for MyHook {
///     const NAME: &'static str = "MyHook";
///     type Output = ();
///  }
///
///  struct MyScript;
///  impl MyScript {
///     fn run(&mut self, hook: &str) {
///         let _hook: MyHook = Self::read();
///         eprintln!("hook: {} was triggered", hook);
///     }
///  }
///  impl Scripter for MyScript {
///     fn name() -> &'static str {
///         "MyScript"
///     }
///     fn script_type() -> ScriptType {
///         ScriptType::OneShot
///     }
///     fn hooks() -> &'static [&'static str] {
///         &[MyHook::NAME]
///     }
///     fn version_requirement() -> VersionReq {
///         VersionReq::parse(">=0.1.0").expect("version requirement is correct")
///     }
///  }
///
///  fn main() {
///     let mut my_script = MyScript;
///     MyScript::execute(&mut |hook_name|MyScript::run(&mut my_script, hook_name));
///  }
pub trait Scripter {
    // Required methods
    /// The name of the script
    fn name() -> &'static str;
    /// The script type Daemon/OneShot
    fn script_type() -> ScriptType;
    /// The hooks that the script is interested in
    fn hooks() -> &'static [&'static str];
    /// The version requirement of the program that the script will run against, when running the script with [Scripter::execute] it will use this version to check if there is an incompatibility between the script and the program
    fn version_requirement() -> VersionReq;

    // Provided methods
    /// Convenient method to read a hook from stdin
    fn read<H: Hook>() -> H {
        bincode::deserialize_from(std::io::stdin()).unwrap()
    }
    /// Convenient method to write a value to stdout
    fn write<T: serde::Serialize>(value: &T) {
        bincode::serialize_into(std::io::stdout(), value).unwrap()
    }
    /// This function is the script entry point.\
    /// 1. It handles receiving [Message::Greeting] , responding with a [ScriptInfo] and exiting if the script type is [ScriptType::OneShot]
    /// 2. It handles receiving hooks, the user is expected to provide a function that acts on a hook name, the user function should use the hook name to read the actual hook from stdin
    ///
    /// Example of a user function:
    /// ```rust
    /// # use rscript::{VersionReq, Hook};
    /// # use rscript::scripting::Scripter;
    /// # #[derive(serde::Serialize, serde::Deserialize)]
    /// # struct MyHook{}
    /// # impl Hook for MyHook {
    /// #   const NAME: &'static str = "MyHook";
    /// #   type Output = usize;
    /// # }
    /// # struct MyScript;
    /// # impl Scripter for MyScript {
    /// #   fn name() -> &'static str { todo!() }
    /// #   fn script_type() -> rscript::ScriptType { todo!() }
    /// #   fn hooks() -> &'static [&'static str] { todo!() }
    /// #   fn version_requirement() -> VersionReq { todo!() }
    /// # }
    ///
    /// fn run(hook_name: &str) {
    ///     match hook_name {
    ///         MyHook::NAME => {
    ///             let hook: MyHook = MyScript::read();
    ///             /*handle hook*/
    ///         }
    ///         _ => unreachable!()
    ///     }
    /// }
    fn execute(func: &mut dyn FnMut(&str)) {
        // 1 - Handle greeting
        let mut stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        let message: Message = bincode::deserialize_from(&mut stdin).unwrap();

        if message == Message::Greeting {
            let metadata = ScriptInfo::new(
                Self::name(),
                Self::script_type(),
                Self::hooks(),
                Self::version_requirement(),
            );
            bincode::serialize_into(&mut stdout, &metadata).unwrap();
            stdout.flush().unwrap();

            // if the script is OneShot it should exit, it will be run again but with message == [Message::Execute]
            if matches!(Self::script_type(), ScriptType::OneShot) {
                std::process::exit(0);
            }
        } else {
            // message == Message::Execute
            // the script will continue its execution
        }

        // 2 - Handle Executing
        loop {
            // OneShot scripts handles greeting each time they are run, so [Message] is already received
            if matches!(Self::script_type(), ScriptType::Daemon) {
                let _message: Message = bincode::deserialize_from(&mut stdin).unwrap();
            }

            let hook_name: String = bincode::deserialize_from(&mut stdin).unwrap();

            func(&hook_name);
            std::io::stdout().flush().unwrap();

            if matches!(Self::script_type(), ScriptType::OneShot) {
                // if its OneShot we exit after one execution
                return;
            }
        }
    }
    /// Check at compile time that the script output matches the output expected by the provided hook
    fn script_static_assert<H: Hook>(_output: &<H as Hook>::Output) {}
}
