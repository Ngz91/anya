use tokio::{
    select,
    sync::{mpsc, watch},
};

use crate::errors;

pub async fn listen_requests(
    mut done: watch::Receiver<bool>,
    mut request_rx: mpsc::UnboundedReceiver<reqwest::RequestBuilder>,
    response_tx: mpsc::UnboundedSender<Result<serde_json::Value, errors::CustomError>>,
) {
    loop {
        select! {
            _ = done.changed() => {
                break
            }
            builder = request_rx.recv() => {
                match builder {
                    None => {
                        break
                    }
                    Some(builder) => {
                        let response = make_request(builder).await;
                        let _ = response_tx.send(response);
                    }
                }
            }
        }
    }
}

pub async fn make_request(
    request_builder: reqwest::RequestBuilder,
) -> std::result::Result<serde_json::Value, errors::CustomError> {
    let resp = request_builder
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(resp)
}
