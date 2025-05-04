use std::sync::Arc;
use whisper_rs::{WhisperContext, WhisperError, WhisperState};

pub struct AppState {
    whisper: Arc<WhisperContext>,
}
impl AppState {
    pub fn new(whisper: Arc<WhisperContext>) -> Self {
        AppState { whisper }
    }
    pub fn state(&self) -> Result<WhisperState, WhisperError> {
        self.whisper.create_state()
    }
}
