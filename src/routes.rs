use std::error::Error;
use std::fs::read;

use actix_web::web::{self, Html};
use actix_web::{HttpRequest, HttpResponse, Responder, get, rt};
use actix_ws::AggregatedMessage;
use askama::Template;
use futures_util::StreamExt as _;
use symphonia::core::formats::FormatReader;
use symphonia::default::codecs::PcmDecoder;
use symphonia::default::formats::MkvReader;
use symphonia::core::io::MediaSourceStream;

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
                    tracing::debug!("Reveived bytes");
                    print!("{:?}", bin);
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

fn demux_audio(data: actix_web::web::Bytes) -> Result<(), String>{
    let cursor = Box::new(std::io::Cursor::new(data));
    let mss = MediaSourceStream::new(cursor, Default::default());
    let reader = match MkvReader::try_new(mss, &Default::default()) {
        Ok(reader) => reader,
        Err(err) => return Err(format!("Invalid audio format: {err}"))
    };
    let track = reader.default_track();
    Ok(())
}
