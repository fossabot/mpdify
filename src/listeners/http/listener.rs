use crate::listeners::http::responses::*;
use crate::mpd_protocol::{Command, HandlerError, HandlerInput, HandlerOutput, HandlerResult};
use crate::util::Settings;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Server};
use log::debug;
use std::net::SocketAddr;
use std::str::Split;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

#[derive(Clone, Debug)]
struct State {
    handler: Arc<Sender<HandlerInput>>,
    auth_path: Arc<String>,
}

pub struct HttpListener {
    address: SocketAddr,
    state: State,
}

impl HttpListener {
    pub fn new(settings: &Settings, handler: Sender<HandlerInput>) -> Self {
        Self {
            address: settings.http_address(),
            state: State {
                handler: Arc::new(handler),
                auth_path: Arc::new(settings.auth_path()),
            },
        }
    }

    pub async fn run(&mut self) {
        let s = self.state.clone();
        let new_service = make_service_fn(move |_| {
            let s = s.clone();
            async { Ok::<_, GenericError>(service_fn(move |req| handle_request(req, s.clone()))) }
        });

        let server = Server::bind(&self.address).serve(new_service);
        self.address = server.local_addr();
        debug!["Listening on http://{}", &self.address];
        server.await.unwrap();
    }

    pub fn get_address(&self) -> String {
        self.address.to_string()
    }
}

async fn handle_request(req: Request<Body>, state: State) -> Result {
    if !req.uri().path().starts_with('/') {
        return not_found();
    }
    let mut path_parts = req.uri().path()[1..].split('/');

    match match path_parts.next() {
        Some("command") => handle_command(state, path_parts).await,
        Some("auth") => handle_auth(req, state).await,
        _ => not_found(),
    } {
        Ok(result) => Ok(result),
        Err(err) => handle_error(err),
    }
}

async fn handle_command(state: State, input: Split<'_, char>) -> Result {
    let command = Command::from_tokens(input)?;
    match execute_command(state, command).await? {
        HandlerOutput::Data(data) => ok_json(&data),
        _ => ok_empty(),
    }
}

async fn handle_auth(req: Request<Body>, state: State) -> Result {
    match req.uri().query() {
        None => {
            let response = execute_command(state, Command::SpotifyAuth(None)).await;
            // Redirect user if we need to authenticate
            if let Err(HandlerError::AuthNeeded(destination)) = &response {
                return auth_redirect(destination);
            };
            // Bubble up unexpected errors
            response?;
            // Otherwise, return an empty 204
            auth_ok()
        }

        Some(_) => {
            // Redirected with a token
            let url = format![
                "{}?{}",
                state.auth_path,
                req.uri().query().unwrap_or_default()
            ];
            debug!["{}", url];
            execute_command(state, Command::SpotifyAuth(Some(url))).await?;
            auth_ok()
        }
    }
}

async fn execute_command(state: State, command: Command) -> HandlerResult {
    let (tx, rx) = oneshot::channel();
    state
        .handler
        .send(HandlerInput { command, resp: tx })
        .await?;
    rx.await.unwrap()
}
