use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

pub fn create_stt_model(path: String) -> Result<WhisperContext, String> {
    let mut config = WhisperContextParameters::new();
    config.use_gpu(true).flash_attn(true).gpu_device(0);
    match WhisperContext::new_with_params(&path, config) {
        Ok(context) => Ok(context),
        Err(err) => Err(format!("Failed to load model: {}", err)),
    }
}

fn run_stt_model(mut state: WhisperState) -> Result<String, String> {
    let strategy = SamplingStrategy::Greedy { best_of: 1 };
    let mut params = FullParams::new(strategy);
    params.set_print_progress(false);
    params.set_print_timestamps(false);

    let result = state.full(params, &[0.0; 10]);
    match result {
        Err(err) => Err(format!("Failed to transcribe: {}", err)),
        Ok(_) => {
            if let Ok(n) = state.full_n_segments() {
                let texts = (0..n).map(|i| {
                    let segment = state.full_get_segment_text(i).unwrap_or(String::from(""));
                    String::from(segment.trim())
                });
                Ok(texts.fold(String::from(""), |acc, s| format!("{acc} {s}")))
            } else {
                Err(String::from("Failed to fetch segments"))
            }
        }
    }
}
