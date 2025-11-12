use axum::{
    response::sse::{Event, KeepAlive, Sse},
    extract::State,
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::StreamExt as _;
use tokio::sync::broadcast;
use serde_json::json;
use tracing::info;

use super::http::AppState;

// Global event broadcaster
lazy_static::lazy_static! {
    static ref EVENT_BROADCASTER: broadcast::Sender<CastEvent> = {
        let (tx, _) = broadcast::channel(100);
        tx
    };
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CastEvent {
    DisplayChanged {
        display_id: String,
        action: String,
    },
    CastStarted {
        display_id: String,
        content_type: String,
        session_id: String,
    },
    CastStopped {
        display_id: String,
        session_id: String,
    },
    ChromecastDiscovered {
        devices: Vec<serde_json::Value>,
    },
    ReceiverStarted {
        protocols: Vec<String>,
        port: u16,
    },
    Error {
        message: String,
    },
}

pub async fn sse_handler(
    State(_state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("New SSE client connected");
    
    let rx = EVENT_BROADCASTER.subscribe();
    
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
        .map(|result| {
            match result {
                Ok(event) => {
                    let json = serde_json::to_string(&event).unwrap_or_default();
                    Ok(Event::default()
                        .event("cast-event")
                        .data(json))
                },
                Err(_) => {
                    // Client lagged, send a sync event
                    Ok(Event::default()
                        .event("sync-required")
                        .data("{}"))
                }
            }
        });
    
    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(30))
                .text("keep-alive")
        )
}

pub fn broadcast_event(event: CastEvent) {
    let _ = EVENT_BROADCASTER.send(event);
}

// Helper functions for common events
pub fn notify_cast_started(display_id: String, content_type: String, session_id: String) {
    broadcast_event(CastEvent::CastStarted {
        display_id,
        content_type,
        session_id,
    });
}

pub fn notify_cast_stopped(display_id: String, session_id: String) {
    broadcast_event(CastEvent::CastStopped {
        display_id,
        session_id,
    });
}

pub fn notify_error(message: String) {
    broadcast_event(CastEvent::Error { message });
}