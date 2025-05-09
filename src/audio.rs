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

pub fn run_stt_model(state: &mut WhisperState, data: Vec<f32>) -> Result<String, String> {
    tracing::info!("Running inference session.");
    let strategy = SamplingStrategy::Greedy { best_of: 1 };
    let mut params = FullParams::new(strategy);
    params.set_print_progress(false);
    params.set_print_timestamps(false);

    let result = state.full(params, &data);
    match result {
        Err(err) => Err(format!("Failed to transcribe: {}", err)),
        Ok(_) => {
            tracing::info!("Succesful transcription.");
            if let Ok(n) = state.full_n_segments() {
                tracing::info!("Retrieving text segments from session.");
                let texts = (0..n).map(|i| {
                    let segment = state.full_get_segment_text(i).unwrap_or(String::from(""));
                    String::from(segment.trim())
                });
                Ok(texts.fold(String::from(""), |acc, s| format!("{acc} {s}")))
            } else {
                tracing::error!("Failed to fetch segments from model.");
                Err(String::from("Failed to fetch segments"))
            }
        }
    }
}
