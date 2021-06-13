use std::process::Stdio;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait HookT: Serialize {
    type O: DeserializeOwned;
    fn trigger(&self) -> Self::O {
        let mut p = std::process::Command::new("/home/mrcool/dev/rust/rscript/script_test")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let i = p.stdin.as_mut().unwrap();
        let o = p.stdout.as_mut().unwrap();
        // bincode write hook
        bincode::serialize_into(i, self).unwrap();
        // bincode read result -> O
        bincode::deserialize_from(o).unwrap()
    }
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
        let x: () = BeforeAssign.trigger();
        dbg!(&x);
        let a = 4;
        let _: () = AfterAssign.trigger();

        if let Some(result) = BeforeEval(a).trigger() {
            println!("{}", result);
        }
        /*eval(a);*/
        print!("{}", a);
    }
}
