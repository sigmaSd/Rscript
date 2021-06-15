use super::{Message, ScriptInfo, ScriptType};
use std::io::Write;

/// Trait that can be implemented on a script abstraction struct\
/// The implementer should provide [Scripter::script_type] [Scripter::name] and [Scripter::hooks]\
///  The struct should call [Scripter::greet] then [Scripter::execute]\
///  ```rust, ignore
///  # // FIXME: rscript::scripting -> erros with could not find scripting
///  # use rscript::*;
///
///  struct MyHook;
///  impl Hook for MyHook{}
///
///  struct MyScript;
///  impl MyScript {
///     fn run(hook: &str) {
///         println!("hook: {} was triggered", hook);
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
///  }
///
///  fn main() {
///     let my_script = MyScript;
///     MyScript::greet();
///     MyScript::execute(|hook_name|MyScript::run(&mut my_script, hook_name));
///  }
pub trait Scripter {
    /// The name of the script
    fn name() -> &'static str;
    /// The script type Daemon/OneShot
    fn script_type() -> ScriptType;
    /// The hooks that the script is interested in
    fn hooks() -> &'static [&'static str];

    /// This function should be called at the start, it will handle receving [Message::Greeting] , responding with a [ScriptInfo] and exiting if the script type is [ScriptType::OneShot]
    fn greet() {
        let mut stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        let message: Message = bincode::deserialize_from(&mut stdin).unwrap();

        assert_eq!(message, Message::Greeting);

        let metadata = ScriptInfo::new(Self::name(), Self::script_type(), Self::hooks());
        bincode::serialize_into(&mut stdout, &metadata).unwrap();
        stdout.flush().unwrap();

        if matches!(Self::script_type(), ScriptType::OneShot) {
            std::process::exit(0);
        }
    }
    /// This function will handle receiving hooks, the user is expected to provide a function that acts on a hook name, the user function should use the hook name to read the actual hook from stdin\
    /// example of a user function:
    /// ```rust
    /// # use rscript::Hook;
    /// # #[derive(serde::Serialize, serde::Deserialize)]
    /// # struct MyHook{}
    /// # impl Hook for MyHook {
    /// #   const NAME: &'static str = "MyHook";
    /// #   type Output = usize;
    /// # }
    ///
    /// fn run(hook_name: &str) {
    ///     match hook_name {
    ///         MyHook::NAME => {
    ///             let hook: MyHook = bincode::deserialize_from(std::io::stdin()).unwrap();
    ///             /*handle hook*/
    ///         }
    ///         _ => unreachable!()
    ///     }
    /// }
    fn execute(func: &mut dyn FnMut(&str)) {
        let mut stdin = std::io::stdin();

        loop {
            let _message: Message = bincode::deserialize_from(&mut stdin).unwrap();
            let hook_name: String = bincode::deserialize_from(&mut stdin).unwrap();

            func(&hook_name);
            std::io::stdout().flush().unwrap();

            if matches!(Self::script_type(), ScriptType::OneShot) {
                // if its OneShot we exit after one execution
                return;
            }
        }
    }
}
