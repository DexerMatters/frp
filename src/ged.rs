use serde::Deserialize;

use crate::signal::SignalRef;

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
    pub x: SignalRef<i32>,
    pub y: SignalRef<i32>,
    pub button: SignalRef<i32>,
    pub name: SignalRef<String>,
}

#[derive(Default)]
pub struct KeyboardSignals {
    pub key: SignalRef<String>,
}

#[derive(Default)]
pub struct InputSignals {
    pub mouse: MouseSignals,
    pub keyboard: KeyboardSignals,
}
