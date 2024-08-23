use std::{future::IntoFuture, sync::Arc};

use futures::join;
use ged::InputSignals;
use signal::Signal;

pub mod api;
pub mod ged;
pub mod signal;

#[tokio::main]
async fn main() {
    let ged = Arc::new(InputSignals::default());
    let app = api::init_router(ged.clone());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let _ = join!(run(ged.clone()), axum::serve(listener, app).into_future());
}

async fn run(ged: Arc<InputSignals>) {
    let mut print = Signal::effect("".to_string(), |new, _| {
        println!("Effect: {}", new);
    });

    ged.mouse
        .x
        .bind(&mut print, |x| format!("Mouse X: {}", x))
        .await;
}
