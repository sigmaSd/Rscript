use rscript::{
    scripting::{DynamicScript, FFiData, FFiStr},
    Hook, ScriptInfo, VersionReq,
};

#[no_mangle]
pub static SCRIPT: DynamicScript = DynamicScript {
    script_info,
    script,
};

pub extern "C" fn script_info() -> FFiData {
    let metadata = ScriptInfo::new(
        "Echo",
        rscript::ScriptType::DynamicLib,
        &[shell_api::Eval::NAME, shell_api::Shutdown::NAME],
        VersionReq::parse(">=0.1.0").expect("correct version requirement"),
    );
    metadata.into_ffi_data()
}

pub extern "C" fn script(name: FFiStr, hook: FFiData) -> FFiData {
    match name.as_str() {
        shell_api::Eval::NAME => {
            let hook: shell_api::Eval = DynamicScript::read(hook);
            let output = hook.0;
            DynamicScript::write::<shell_api::Eval>(&output)
        }
        shell_api::Shutdown::NAME => {
            eprintln!("bye from hello-script");
            DynamicScript::write::<shell_api::Shutdown>(&())
        }
        _ => unreachable!(),
    }
}
