*0.6.0*
- Add 2 new functions to the public API, [Scripter::read] [Scripter::write], these functions are convenient methods to read hooks from stdin and write a value to stdout respectively
- Trait Hook now requires [serde::de::DeserializeOwned]
- Add an example of using Rscript

*0.5.0*
- Remove [scripting::Scripter::greet] from the public API, the user is now only required to use [scripting::Scripter::execute] which will handle the greeting.
- Improve the documentation

*0.4.0*
- Fix oneshot script crashing in scripting::greet

*0.3.0*
Add 2 new public API methods: 
- `Script::is_listening_for` -> check if a script is listening for a hook 
- `Script::trigger` -> triggers a script regardless of its state (active/inactive) if the script is not listening for the specified hook an error will be returned

*0.2.0*
- Add `scripting` module, this module provides utilities that improves writing scripts experience
- Derive `Copy/Clone` on `ScriptType`

*0.1.0*
- Initial release
