use serde::Deserialize;

use crate::signal::SignalArc;

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
    pub x: SignalArc<i32>,
    pub y: SignalArc<i32>,
    pub button: SignalArc<i32>,
    pub name: SignalArc<String>,
}

#[derive(Default)]
pub struct KeyboardSignals {
    pub key: SignalArc<String>,
}

#[derive(Default)]
pub struct InputSignals {
    pub mouse: MouseSignals,
    pub keyboard: KeyboardSignals,
}
