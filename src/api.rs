use std::sync::Arc;

use axum::{debug_handler, extract::State, routing::post, Json, Router};
use futures::join;
use tower_http::services::ServeDir;

use crate::{
    ged::{InputSignals, MouseEvent},
    signal::Signal,
};

pub fn init_router(inputs: Arc<InputSignals>) -> Router {
    Router::new()
        .nest_service("/", ServeDir::new("public"))
        .route("/mouse-event", post(mouse_event))
        .with_state(inputs)
}

#[debug_handler]
async fn mouse_event(signals: State<Arc<InputSignals>>, Json(event_body): Json<MouseEvent>) {
    let mouse = &signals.mouse;
    dbg!("Mouse Event");
    join!(
        Signal::change(&mouse.x, event_body.x),
        Signal::change(&mouse.y, event_body.y),
        Signal::change(&mouse.button, event_body.button),
        Signal::change(&mouse.name, event_body.name),
    );
}
