# Rscript examples

## shell

This crate is  structured like:

```
shell---shell-main (The main binary)
      |
      |-shell-api  (The common api)
      |
      |-scripts    (The scritps directory)
```


To test this example:
1. Enter shell directory `cd shell`
2. Compile the workspace `cargo b`
3. The main crate expects the scripts to be in `/tmp/rscript_shell` (or the platform equivalent), so we create it `mkdir /tmp/rscript_shell` and  we can just cp the scripts or symlink it  `ln -s target/debug/eval-script /tmp/rscript_shell` `ln -s target/debug/random-script /tmp/rscript_shell`
4. Now we can run the main binary `cargo r --bin shell-main`, you can try inputing some random command (`ls` for example) and hit enter.

