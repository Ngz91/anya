use tokio::{
    select,
    sync::{mpsc, watch},
};

use crate::local_types::ResultSerde;

pub async fn listen_requests(
    mut done: watch::Receiver<bool>,
    mut request_rx: mpsc::UnboundedReceiver<reqwest::RequestBuilder>,
    response_tx: mpsc::UnboundedSender<ResultSerde>,
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
                    Some(b) => {
                        let response = make_request(b).await;
                        let _ = response_tx.send(response);
                    }
                }
            }
        }
    }
}

pub async fn make_request(request_builder: reqwest::RequestBuilder) -> ResultSerde {
    let resp = request_builder
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(resp)
}
