use std::{
    io,
    path::Path,
    process::{Child, Stdio},
};

use serde::{de::DeserializeOwned, Serialize};

#[derive(Default)]
pub struct ScriptManager {
    scripts: Vec<Script>,
}
impl ScriptManager {
    pub fn add_scripts_by_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), io::Error> {
        fn start_script(path: &Path) -> Result<Script, io::Error> {
            let script = std::process::Command::new(path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;
            Ok(Script(script))
        }

        let path = path.as_ref();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                self.scripts.push(start_script(&path)?);
            }
        }
        Ok(())
    }
    pub fn trigger<'a, H: 'static + HookT>(
        &'a mut self,
        hook: H,
    ) -> impl Iterator<Item = Result<<H as HookT>::O, bincode::Error>> + 'a {
        self.scripts
            .iter_mut()
            .map(move |script| script.trigger(&hook))
    }
}

impl Drop for ScriptManager {
    fn drop(&mut self) {
        self.scripts.iter_mut().for_each(|script| script.end());
    }
}

struct Script(Child);
impl Script {
    fn trigger<H: HookT>(&mut self, hook: &H) -> Result<<H as HookT>::O, bincode::Error> {
        let stdin = self.0.stdin.as_mut().expect("stdin is piped");
        let stdout = self.0.stdout.as_mut().expect("stdout is piped");
        // bincode write hook
        bincode::serialize_into(stdin, hook)?;
        // bincode read result -> O
        bincode::deserialize_from(stdout)
    }
    fn end(&mut self) {
        // This errors if the script has already exited
        // We don't care about this error
        let _ = self.0.kill();
    }
}

pub trait HookT: Serialize {
    type O: DeserializeOwned;
}

#[cfg(test)]
mod tests {
    use super::*;

    //hooks
    #[derive(Serialize)]
    struct BeforeAssign;
    impl HookT for BeforeAssign {
        type O = ();
    }
    #[derive(Serialize)]
    struct AfterAssign;
    impl HookT for AfterAssign {
        type O = ();
    }
    #[derive(Serialize)]
    struct BeforeEval(usize);
    impl HookT for BeforeEval {
        type O = Option<String>;
    }

    #[test]
    fn prototype() {
        let mut sm = ScriptManager::default();
        sm.add_scripts_by_path("scripts").unwrap();
        let _ = sm.trigger(BeforeAssign);
        let a = 4;
        let _ = sm.trigger(AfterAssign);

        if let Some(result) = sm.trigger(BeforeEval(a)).next().unwrap().unwrap() {
            println!("{}", result);
        }
        /*eval(a);*/
        print!("{}", a);
    }
}
