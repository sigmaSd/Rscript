use rscript::{Hook, ScriptInfo, VersionReq};

#[no_mangle]
pub fn script_info() -> ScriptInfo {
    ScriptInfo::new(
        "Echo",
        rscript::ScriptType::DynamicLib,
        &[shell_api::Eval::NAME, shell_api::Shutdown::NAME],
        VersionReq::parse(">=0.1.0").expect("correct version requirement"),
    )
}

#[no_mangle]
pub fn script(hook: &str, data: Vec<u8>) -> Vec<u8> {
    match hook {
        shell_api::Eval::NAME => {
            let data: shell_api::Eval = bincode::deserialize(&data).unwrap();
            let output = data.0;
            bincode::serialize(&output).unwrap()
        }
        shell_api::Shutdown::NAME => {
            eprintln!("bye from hello-script");
            bincode::serialize(&()).unwrap()
        }
        _ => unreachable!(),
    }
}
