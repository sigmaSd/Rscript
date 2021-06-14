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
