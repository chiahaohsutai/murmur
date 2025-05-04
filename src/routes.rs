use crate::audio::run_stt_model;
use crate::state::AppState;
use actix_web::web::{self, Bytes, Html};
use actix_web::{HttpRequest, HttpResponse, Responder, get, rt};
use actix_ws::AggregatedMessage;
use askama::Template;
use futures_util::StreamExt as _;

#[derive(Debug, Template)]
#[template(path = "index.html")]
struct Index;

#[get("/")]
pub async fn index() -> impl Responder {
    let template = Index;
    match template.render() {
        Ok(html) => Html::new(html),
        Err(err) => Html::new(format!("Error: {err}")),
    }
}

#[get("/ws")]
pub async fn ws(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let (res, mut sess, stream) = actix_ws::handle(&req, stream)?;
    let mut stream = stream.aggregate_continuations();
    tracing::info!("Connection request. Upgrading the socket connection.");

    rt::spawn(async move {
        let mut whisper_state = data.state();
        while let Some(msg) = stream.next().await {
            let mut max_retries = 3;
            while let Err(err) = whisper_state {
                tracing::error!("Failed to create inference session. Retrying ...");
                if max_retries == 0 {
                    tracing::error!("Failed to start a inference session: {err}");
                    break;
                }
                whisper_state = data.state();
                max_retries -= 1;
            }
            if whisper_state.is_err() {
                continue;
            }
            match msg {
                Ok(AggregatedMessage::Binary(bin)) => {
                    tracing::info!("Received Message");
                    let audio = bytes_to_f32_vec(bin);
                    let state = whisper_state.as_mut().unwrap();
                    match run_stt_model(state, audio) {
                        Ok(text) => {
                            if let Err(err) = sess.text(text).await {
                                tracing::error!("Failed to send data to client: {err}")
                            }
                        }
                        Err(err) => {
                            tracing::error!("Failed to transcribe audio: {err}")
                        }
                    }
                }
                _ => {}
            }
        }
        tracing::info!("Closing the socket connection.");
        let _ = sess.close(None).await;
    });
    Ok(res)
}

fn bytes_to_f32_vec(bytes: Bytes) -> Vec<f32> {
    let data = bytes.as_ref();
    assert!(data.len() % 4 == 0, "Byte length must be divisible by 4");

    data.chunks_exact(4)
        .map(|chunk| {
            let array: [u8; 4] = chunk.try_into().unwrap();
            f32::from_le_bytes(array)
        })
        .collect()
}
