use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::{Message, Session};
use serde_json::json;
use std::sync::{Arc, Mutex};
use crate::audio_engine::{AudioEngine, AudioMessage};

pub struct WsAudioSession {
    session: Session,
    engine: Arc<Mutex<AudioEngine>>,
    subscriptions: Vec<Arc<Mutex<()>>>,
}

impl WsAudioSession {
    pub fn new(session: Session, engine: Arc<Mutex<AudioEngine>>) -> Self {
        WsAudioSession {
            session,
            engine,
            subscriptions: Vec::new(),
        }
    }

    pub async fn handle_message(&mut self, msg: Message) -> Result<(), Error> {
        match msg {
            Message::Text(text) => {
                if let Ok(audio_msg) = serde_json::from_str::<AudioMessage>(&text) {
                    let state = {
                        let mut engine = self.engine.lock().unwrap();
                        engine.handle_message(audio_msg)
                    };

                    if let Some(new_state) = state {
                        let response = json!({
                            "type": "state_update",
                            "state": new_state
                        });
                        self.session.text(response.to_string()).await?;
                    }
                }
            }
            Message::Close(reason) => {
                self.session.close(reason).await?;
            }
            Message::Binary(_) => {}
            Message::Ping(bytes) => {
                self.session.pong(&bytes).await?;
            }
            Message::Pong(_) => {}
            Message::Continuation(_) => {}
            Message::Nop => {}
        }
        Ok(())
    }

    pub async fn send_state(&self) -> Result<(), Error> {
        let state = self.engine.lock().unwrap().get_state();
        let response = json!({
            "type": "state_update",
            "state": state
        });
        self.session.text(response.to_string()).await?;
        Ok(())
    }
}

pub async fn audio_ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    engine: web::Data<Arc<Mutex<AudioEngine>>>,
) -> Result<HttpResponse, Error> {
    let (session, response) = actix_ws::handle(&req, stream)?;

    let engine = engine.get_ref().clone();
    let mut ws_session = WsAudioSession::new(session, engine);

    let subscription = {
        let mut engine = engine.lock().unwrap();
        engine.subscribe()
    };
    ws_session.subscriptions.push(subscription);

    ws_session.send_state().await?;

    actix_ws::session::handle_session(ws_session, response)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/ws/audio", web::get().to(audio_ws_handler));
}
