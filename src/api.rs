use std::sync::Arc;

use axum::{extract::State, handler::Handler, routing::post, Json, Router, ServiceExt};
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
    let mut x = mouse.x.lock().await;
    let mut y = mouse.y.lock().await;
    let mut button = mouse.button.lock().await;
    let mut name = mouse.name.lock().await;
    dbg!("Mouse Event");
    join!(
        x.change(event_body.x),
        y.change(event_body.y),
        button.change(event_body.button),
        name.change(event_body.name),
    );
}
