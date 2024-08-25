use std::{future::IntoFuture, sync::Arc};

use futures::join;
use ged::InputSignals;

pub mod api;
pub mod ged;
pub mod signal;
pub mod signals;

#[tokio::main]
async fn main() {
    let ged = Arc::new(InputSignals::default());
    let app = api::init_router(ged.clone());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let _ = join!(run(ged.clone()), axum::serve(listener, app).into_future());
}

async fn run(ged: Arc<InputSignals>) {
    (ged.mouse.x.clone())
        // ^ signal of the x coordinate of the mouse
        .with(ged.mouse.y.clone())
        // ^ Bind the signal of the y coordinate of the mouse
        .with(ged.mouse.name.clone())
        // ^ Bind the signal of the name of the mouse event
        .with_guard(|(_, name)| *name == "mousedown")
        // ^ Filter the signal except for the mousedown event
        .map(|(p, _)| *p)
        // ^ Extract the x and y coordinates from the signal
        .with_effect(|(x, y), _| {
            println!("Mouse Down at ({}, {})", x, y);
            true
        });
}
