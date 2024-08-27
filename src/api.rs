use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use futures::join;
use tower_http::services::ServeDir;

use crate::ged::{InputSignals, MouseEvent};

pub fn init_router(inputs: Arc<InputSignals>) -> Router {
    Router::new()
        .nest_service("/", ServeDir::new("public"))
        .route("/mouse-event", post(mouse_event))
        .with_state(inputs)
}

async fn mouse_event(signals: State<Arc<InputSignals>>, Json(event_body): Json<MouseEvent>) {
    let mouse = &signals.mouse;
    join!(
        mouse.x.clone().update(event_body.x),
        mouse.y.clone().update(event_body.y),
        mouse.name.clone().update(event_body.name)
    );
}
