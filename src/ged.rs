use serde::Deserialize;

use crate::signal::Signal;

#[derive(Deserialize, Debug)]
pub struct MouseEvent {
    pub x: i32,
    pub y: i32,
    pub button: i32,
    pub name: String,
}

pub struct KeyboardEvent {
    pub key: String,
}

#[derive(Default)]
pub struct MouseSignals {
    pub x: Signal<i32>,
    pub y: Signal<i32>,
    pub button: Signal<i32>,
    pub name: Signal<String>,
}

#[derive(Default)]
pub struct KeyboardSignals {
    pub key: Signal<String>,
}

#[derive(Default)]
pub struct InputSignals {
    pub mouse: MouseSignals,
    pub keyboard: KeyboardSignals,
}
