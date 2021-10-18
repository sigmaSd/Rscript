use rscript::{FFiVec, Hook, ScriptInfo, VersionReq};

#[no_mangle]
pub extern "C" fn script_info() -> FFiVec {
    let metadata = ScriptInfo::new(
        "Echo",
        rscript::ScriptType::DynamicLib,
        &[shell_api::Eval::NAME, shell_api::Shutdown::NAME],
        VersionReq::parse(">=0.1.0").expect("correct version requirement"),
    );
    FFiVec::serialize_from(&metadata).unwrap()
}

#[no_mangle]
pub extern "C" fn script(hook: FFiVec, data: FFiVec) -> FFiVec {
    let hook: String = hook.deserialize().unwrap();

    match hook.as_str() {
        shell_api::Eval::NAME => {
            let data: shell_api::Eval = data.deserialize().unwrap();
            let output = data.0;
            FFiVec::serialize_from(&output).unwrap()
        }
        shell_api::Shutdown::NAME => {
            eprintln!("bye from hello-script");
            FFiVec::serialize_from(&()).unwrap()
        }
        _ => unreachable!(),
    }
}
