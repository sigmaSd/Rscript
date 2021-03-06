use std::time::SystemTime;

use rscript::{scripting::Scripter, Hook, VersionReq};

struct Randomize;
impl Scripter for Randomize {
    fn name() -> &'static str {
        "randomize"
    }

    fn script_type() -> rscript::ScriptType {
        rscript::ScriptType::Daemon
    }

    fn hooks() -> &'static [&'static str] {
        &[shell_api::RandomNumber::NAME]
    }

    fn version_requirement() -> VersionReq {
        VersionReq::parse(">=0.1.0").expect("correct version requirement")
    }
}

impl Randomize {
    fn run(&self, hook: &str) {
        match hook {
            shell_api::RandomNumber::NAME => {
                let _hook: shell_api::RandomNumber = Self::read();
                let output: usize = Self::random();
                Self::write::<shell_api::RandomNumber>(&output);
            }
            _ => unreachable!(),
        }
    }
    fn random() -> usize {
        let num = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros()
            % 100;

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        num.hash(&mut hasher);
        hasher.finish() as usize % 100
    }
}

fn main() {
    let randomize = Randomize;
    Randomize::execute(&mut |hook| {
        randomize.run(hook);
    }).unwrap();
}
