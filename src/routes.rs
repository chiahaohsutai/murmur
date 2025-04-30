use actix_web::web::{self, Html};
use actix_web::{HttpRequest, HttpResponse, Responder, get, rt};
use actix_ws::AggregatedMessage;
use askama::Template;
use futures_util::StreamExt as _;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::errors::Error::DecodeError;
use symphonia::core::formats::FormatReader;
use symphonia::core::io::MediaSourceStream;
use symphonia::default::formats::MkvReader;
use symphonia::default::get_codecs;
use crate::audio::run_stt_model;

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
pub async fn ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    let (res, mut sess, stream) = actix_ws::handle(&req, stream)?;
    let mut stream = stream.aggregate_continuations();

    tracing::info!("Connection request. Upgrading the socket connection.");

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Binary(bin)) => {
                    // let audio = decode_audio_to_f32(bin);
                    // let text = run_stt_model();
                    sess.binary(bin).await.unwrap();
                }
                _ => {}
            }
        }
        tracing::info!("Closing the socket connection.")
        // let _ = sess.close(None).await;
    });
    Ok(res)
}

fn decode_audio_to_f32(data: actix_web::web::Bytes) -> Result<Vec<f32>, String> {
    let cursor = Box::new(std::io::Cursor::new(data));

    let mss = MediaSourceStream::new(cursor, Default::default());
    let mut reader = match MkvReader::try_new(mss, &Default::default()) {
        Ok(reader) => reader,
        Err(err) => return Err(format!("Invalid audio format: {err}")),
    };

    let track = match reader.default_track() {
        Some(track) => track,
        None => return Err(format!("No audio track was found.")),
    };
    let track_id = track.id;

    let mut decoder = match get_codecs().make(&track.codec_params, &Default::default()) {
        Ok(decoder) => decoder,
        Err(err) => return Err(format!("Unsupported audio encoding: {err}.")),
    };

    let mut buffer = None;
    let mut audio: Vec<f32> = vec![];

    loop {
        let packet = reader.next_packet();
        if let Err(err) = packet {
            return Err(format!("Failed to read audio packet: {err}"));
        }
        let packet = packet.unwrap();
        if packet.track_id() != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                if buffer.is_none() {
                    let duration = audio_buf.capacity() as u64;
                    let spec = *audio_buf.spec();
                    buffer = Some(SampleBuffer::new(duration, spec))
                }
                if let Some(buf) = &mut buffer {
                    buf.copy_interleaved_ref(audio_buf);
                    let samples: &[f32] = buf.samples();
                    samples.iter().for_each(|sample| audio.push(*sample));
                }
            }
            Err(DecodeError(_)) => (),
            Err(_) => break
        }
    }
    Ok(audio)
}
