use std::{future::IntoFuture, sync::Arc};

use futures::join;
use ged::InputSignals;
use signals::pure;

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
    pure(|x: i32| move |y: i32| move |name: String| (x, y, name))
        .apply(ged.mouse.x.clone())
        .apply(ged.mouse.y.clone())
        .apply(ged.mouse.name.clone())
        .with_guard(|(_, _, name), _| name == "mousedown")
        .map(|(x, y, _)| (x, y))
        .with_effect(|(x, y), _| {
            println!("Mouse moved to ({}, {})", x, y);
        });
}
