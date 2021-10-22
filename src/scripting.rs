//! This modules contains all what is needed to write scripts

use crate::{Hook, VersionReq};

use super::{Message, ScriptInfo, ScriptType};
use std::io::Write;
use std::ptr::slice_from_raw_parts;

use serde::{de::DeserializeOwned, Serialize};

/// Trait that should be implemented on a script abstraction struct\
/// This concerns [ScriptType::OneShot] and [ScriptType::Daemon]\
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
    /// Read a hook from stdin
    fn read<H: Hook>() -> H {
        bincode::deserialize_from(std::io::stdin()).unwrap()
    }
    /// Write a value to stdout\
    /// It takes the hook as a type argument in-order to make sure that the output provided correspond to the hook's expected output
    fn write<H: Hook>(output: &<H as Hook>::Output) {
        bincode::serialize_into(std::io::stdout(), output).unwrap()
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
    ///             let output = todo!(); // prepare the corresponding hook output
    ///             MyScript::write::<MyHook>(&output);
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
}

#[repr(C)]
/// A [ScriptType::DynamicLib] script needs to export a static instance of this struct named [DynamicScript::NAME]
/// ```rs
/// // In a script file
/// #[no_mangle]
/// pub static SCRIPT: DynamicScript = DynamicScript { script_info: .., script: .. };
/// ```
///
///
/// `DynamicScript` contains also methods for writing scripts: [DynamicScript::read], [DynamicScript::write]
pub struct DynamicScript {
    /// A function that returns `ScriptInfo` serialized as `FFiData`\
    /// *fn() -> ScriptInfo*
    pub script_info: extern "C" fn() -> FFiData,
    /// A function that accepts a hook name (casted to `FFiStr`) and the hook itself (serialized as `FFiData`)  and returns the hook output (serialized as `FFiData`)\
    /// *fn<H: Hook>(hook: &str (H::Name), data: H) -> <H as Hook>::Output>*
    pub script: extern "C" fn(FFiStr, FFiData) -> FFiData,
}
impl DynamicScript {
    /// ```rust
    /// pub const NAME: &'static [u8] = b"SCRIPT";
    /// ```
    pub const NAME: &'static [u8] = b"SCRIPT";

    /// Read a hook from an FFiData
    pub fn read<H: Hook>(hook: FFiData) -> H {
        hook.deserialize().unwrap()
    }
    /// Write a value to an FFiData
    /// It takes the hook as a type argument in-order to make sure that the output provided correspond to the hook's expected output
    pub fn write<H: Hook>(output: &<H as Hook>::Output) -> FFiData {
        FFiData::serialize_from(output).unwrap()
    }
}

#[repr(C)]
/// `FFiStr` is used to send the hook name to [ScriptType::DynamicLib] script
pub struct FFiStr {
    ptr: *const u8,
    len: usize,
}
impl FFiStr {
    /// Create a `FFiStr` from a `&str`
    pub fn new(string: &'static str) -> Self {
        Self {
            ptr: string as *const str as _,
            len: string.len(),
        }
    }
    /// Cast `FFiStr` to `&str`
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.ptr, self.len)) }
    }
}

/// `FFiData` is used for communicating arbitrary data between [ScriptType::DynamicLib] scripts and the main program
#[repr(C)]
pub struct FFiData {
    ptr: *mut u8,
    len: usize,
    cap: usize,
}
impl FFiData {
    /// Crate a new FFiData from any serialize-able data
    pub(crate) fn serialize_from<D: Serialize>(data: &D) -> Result<Self, bincode::Error> {
        let data = bincode::serialize(data)?;
        let mut vec = std::mem::ManuallyDrop::new(data);
        let ptr = vec.as_mut_ptr();
        let len = vec.len();
        let cap = vec.capacity();
        Ok(FFiData { ptr, len, cap })
    }
    /// De-serialize into a concrete type
    pub(crate) fn deserialize<D: DeserializeOwned>(&self) -> Result<D, bincode::Error> {
        let data: &[u8] = unsafe { &*slice_from_raw_parts(self.ptr, self.len) };
        bincode::deserialize(data)
    }
}
impl Drop for FFiData {
    fn drop(&mut self) {
        let _ = unsafe { Vec::from_raw_parts(self.ptr, self.len, self.cap) };
    }
}
