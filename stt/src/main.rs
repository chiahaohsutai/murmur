use dotenv::dotenv;
use stt::create_stt_model;

fn main() {
    dotenv().ok();
    whisper_rs::install_logging_hooks();

    let _ctx = match std::env::var("STT_MODEL_PATH") {
        Ok(path) => match create_stt_model(path) {
            Ok(context) => context,
            Err(err) => {
                eprint!("Failed to load model into memory: {}", err);
                std::process::exit(1);
            }
        },
        Err(err) => {
            eprint!("Missing STT_MODEL_PATH environment variable: {}", err);
            std::process::exit(1);
        }
    };
}
