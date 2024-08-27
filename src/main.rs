use std::{future::IntoFuture, sync::Arc};

use futures::join;
use ged::InputSignals;
use signal::Signal;

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

async fn run(g: Arc<InputSignals>) {
    // Signal::new(|x: i32| move |y: i32| move |_: String| format!("Output: {}, {}", x, y))
    //     .apply(g.mouse.x.clone())
    //     .apply(g.mouse.y.clone())
    //     .apply(g.mouse.name.clone())
    //     .effect(|text, _| {
    //         println!("{}", text);
    //         true
    //     });

    Signal::new((g.mouse.x.clone().strict(), g.mouse.y.clone().strict()))
        .join()
        .effect(|(x, y), _| {
            println!("Output: {}, {}", x, y);
            true
        });
}
