use rscript::{scripting::Scripter, Hook, VersionReq};

struct Evaluator;
impl Scripter for Evaluator {
    fn name() -> &'static str {
        "evaluator"
    }

    fn script_type() -> rscript::ScriptType {
        rscript::ScriptType::OneShot
    }

    fn hooks() -> &'static [&'static str] {
        &[shell_api::Eval::NAME, shell_api::Shutdown::NAME]
    }
    fn version_requirement() -> VersionReq {
        VersionReq::parse(">=0.1.0").expect("correct version requirement")
    }
}

impl Evaluator {
    fn run(&self, hook: &str) {
        match hook {
            shell_api::Eval::NAME => {
                let eval_hook: shell_api::Eval = Self::read();
                let shell_api::Eval(input) = eval_hook;
                let output: String = self.eval(&input);
                Self::write(&output);
                Self::script_static_assert::<shell_api::Eval>(&output);
            }
            shell_api::Shutdown::NAME => {
                let _eval_hook: shell_api::Shutdown = Self::read();
                // stderr is *not* piped so it can be used by scripts
                eprintln!("bye from shell-script");
                Self::script_static_assert::<shell_api::Shutdown>(&());
            }

            _ => unreachable!(),
        }
    }
    fn eval(&self, input: &str) -> String {
        let mut input = input.split_whitespace();
        String::from_utf8(
            std::process::Command::new(input.next().unwrap())
                .args(input.collect::<Vec<_>>())
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
    }
}

fn main() {
    let evaluator = Evaluator;
    Evaluator::execute(&mut |hook| {
        evaluator.run(hook);
    });
}
