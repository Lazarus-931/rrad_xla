use rrad_pjrt::rrad_pjrt::loader::PjrtRuntime;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let plugin = std::env::var("PJRT_PLUGIN").unwrap_or_else(|_| "libtpu.so".to_string());
    let rt = match PjrtRuntime::load(Path::new(&plugin)) {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("failed to load PJRT plugin '{plugin}': {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = rt.initialize_plugin() {
        eprintln!("failed to initialize PJRT plugin '{plugin}': {err}");
        return ExitCode::FAILURE;
    }

    println!("done: {}", plugin);
    ExitCode::SUCCESS
}
