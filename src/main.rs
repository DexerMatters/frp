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
    let print = Signal::effect("".to_string(), |new, _| {
        println!("Effect: {}", new);
    });

    let aux = Signal::new(0);

    let aux2 = Signal::new((0, 0));

    let _ = join!(ged.mouse.x.map_(&aux, |x| x / 2), aux.print());
}
