use dotenv::dotenv;
use tracing::instrument;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters,
    convert_integer_to_float_audio, convert_stereo_to_mono_audio, install_logging_hooks,
};

#[instrument]
fn load_model(path: String) -> Result<WhisperContext, String> {
    let mut config = WhisperContextParameters::new();
    config.use_gpu(true).flash_attn(true).gpu_device(0);
    match WhisperContext::new_with_params(&path, config) {
        Ok(context) => Ok(context),
        Err(err) => Err(format!("Failed to load model: {}", err)),
    }
}

fn main() {
    dotenv().ok();
    install_logging_hooks();

    let model_path = std::env::var("STT_MODEL_PATH").unwrap();
    let wav_path = std::env::var("STT_SAMPLE_PATH").unwrap();
    let language = "en";

    let samples: Vec<i16> = hound::WavReader::open(wav_path)
        .unwrap()
        .into_samples::<i16>()
        .map(|x| x.unwrap())
        .collect();

    let ctx = load_model(model_path).expect("Failed to load model");
    let mut state = ctx.create_state().expect("Failed to create state");

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(&language));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    let mut inter_samples = vec![Default::default(); samples.len()];
    convert_integer_to_float_audio(&samples, &mut inter_samples)
        .expect("Failed to convert audio data");
    let samples =
        convert_stereo_to_mono_audio(&inter_samples).expect("failed to convert audio data");

    state
        .full(params, &samples[..])
        .expect("Failed to run model");
    let num_segments = state
        .full_n_segments()
        .expect("Failed to get number of segments");

    for i in 0..num_segments {
        let segment = state
            .full_get_segment_text(i)
            .expect("failed to get segment");
        let start_timestamp = state
            .full_get_segment_t0(i)
            .expect("failed to get segment start timestamp");
        let end_timestamp = state
            .full_get_segment_t1(i)
            .expect("failed to get segment end timestamp");
        println!("[{} - {}]: {}", start_timestamp, end_timestamp, segment);
    }
}
