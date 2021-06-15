# rscript

Crate to easily script any rust project
## Rscript
The main idea is:
- Create a new crate (my-project-api for example)
- Add hooks to this api-crate
- This api-crate should be used by the main-crate and by the scripts
- Trigger Hooks in the main crate
- Receive the hooks on the script side, and react to them with any output


Goals:
- Be as easy as possible to include on already established projects
- Strive for maximum compile time guarantees

This crate was extracted from [IRust](https://github.com/sigmaSd/IRust)

Taking *IRust* as an example:
- It has an API crate where hooks are defined [irust_api](https://github.com/sigmaSd/IRust/blob/master/crates/irust_api/src/lib.rs#L22)
- It trigger hooks on the main crate [irust](https://github.com/sigmaSd/IRust/blob/master/crates/irust/src/irust.rs#L136)
- And a script example [vim_mode](https://github.com/sigmaSd/IRust/tree/master/scripts_examples/script4/irust_vim)

License: MIT
