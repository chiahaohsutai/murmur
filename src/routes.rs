use actix_web::web::{self, Html};
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
pub async fn ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    let (res, mut sess, stream) = actix_ws::handle(&req, stream)?;
    let mut stream = stream.aggregate_continuations();

    tracing::info!("Upgrading the socket connection.");

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    tracing::debug!("Reveived a Bytestring");
                    let msg = format!(r#"<span id="transcription" hx-swap-oob="beforeend">Hola </span>"#);
                    sess.text(msg).await.unwrap();
                }
                Ok(AggregatedMessage::Binary(bin)) => {
                    tracing::debug!("Reveived bytes");
                    sess.binary(bin).await.unwrap();
                }
                Ok(AggregatedMessage::Ping(msg)) => {
                    tracing::debug!("Reveived a pong");
                    sess.pong(&msg).await.unwrap();
                }
                _ => {}
            }
        }
        tracing::info!("Closing the socket connection.")
        // let _ = sess.close(None).await;
    });
    Ok(res)
}
