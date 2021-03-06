use rscript::{ScriptManager, Version};

/// Simple try macros to ignore errors
macro_rules! mtry {
    ($e: expr) => {
        (|| -> Option<()> { Some($e) })()
    };
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let mut script_manager = ScriptManager::default();
    // FIXME: Auto compile instead
    let scripts_path = std::env::temp_dir().join("rscript_shell");
    let _ = std::fs::create_dir_all(&scripts_path);

    // Add executable scripts
    script_manager
        .add_scripts_by_path(
            &scripts_path,
            Version::parse(VERSION).expect("version is correct"),
        )
        .unwrap();
    // Add dynamic library scripts
    unsafe {
        script_manager
            .add_dynamic_scripts_by_path(
                &scripts_path,
                Version::parse(VERSION).expect("version is correct"),
            )
            .unwrap();
    }

    loop {
        let input = {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            input
        };
        if input.trim() == ":q" {
            break;
        }

        // Many scripts can react to the same hook => collect all results
        let outputs: Vec<_> = script_manager.trigger(shell_api::Eval(input)).collect();
        for out in outputs {
            println!("Out: {:?}", out);
        }

        let _ = mtry!({
            // Many scripts can react to the same hook, we will just use the first one's response
            let num = script_manager
                .trigger(shell_api::RandomNumber)
                .next()?
                .ok()?;
            println!("Random number is {}", &num);
        });
    }

    // Give a chance for all listening scripts to cleanup
    script_manager
        .trigger(shell_api::Shutdown)
        .for_each(|_result| {});
}
