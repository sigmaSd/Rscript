use rscript::Hook;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Eval(pub String);
impl Hook for Eval {
    const NAME: &'static str = "Eval";

    type Output = String;
}

#[derive(Serialize, Deserialize)]
pub struct Shutdown;
impl Hook for Shutdown {
    const NAME: &'static str = "Shutdown";

    type Output = ();
}

#[derive(Serialize, Deserialize)]
pub struct RandomNumber;
impl Hook for RandomNumber {
    const NAME: &'static str = "RandomNumber";

    type Output = usize;
}
