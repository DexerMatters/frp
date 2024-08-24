use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use tower_http::services::ServeDir;

use crate::{
    ged::{InputSignals, MouseEvent},
    signal::Signalable,
};

pub fn init_router(inputs: Arc<InputSignals>) -> Router {
    Router::new()
        .nest_service("/", ServeDir::new("public"))
        .route("/mouse-event", post(mouse_event))
        .with_state(inputs)
}

async fn mouse_event(signals: State<Arc<InputSignals>>, Json(event_body): Json<MouseEvent>) {
    let mouse = &signals.mouse;
    mouse.x.replace(event_body.x);
    mouse.y.replace(event_body.y);
    mouse.button.replace(event_body.button);
    mouse.name.replace(event_body.name);
}
