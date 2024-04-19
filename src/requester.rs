use tokio::sync::mpsc;

use crate::errors;

use crate::State;

pub struct Requester {
    state_rx: mpsc::Receiver<State>,
    request_rx: mpsc::Receiver<reqwest::RequestBuilder>,
    response_tx: mpsc::Sender<Result<serde_json::Value, errors::CustomError>>,
    last_request: Option<reqwest::RequestBuilder>,
}

impl Requester {
    pub fn new(
        state_rx: mpsc::Receiver<State>,
        request_rx: mpsc::Receiver<reqwest::RequestBuilder>,
        response_tx: mpsc::Sender<Result<serde_json::Value, errors::CustomError>>,
        last_request: Option<reqwest::RequestBuilder>,
    ) -> Self {
        Requester {
            state_rx,
            request_rx,
            response_tx,
            last_request,
        }
    }

    pub async fn start_requester(&mut self) {
        println!("Requester initialized");
        while self.state_rx.recv().await.unwrap() != State::Exit {
            println!("Requester Here")
        }
        println!("Requester Shutdown");
    }
}
