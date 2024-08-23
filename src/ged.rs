use serde::Deserialize;

use crate::signal::SignalLock;

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
    pub x: SignalLock<i32>,
    pub y: SignalLock<i32>,
    pub button: SignalLock<i32>,
    pub name: SignalLock<String>,
}

#[derive(Default)]
pub struct KeyboardSignals {
    pub key: SignalLock<String>,
}

#[derive(Default)]
pub struct InputSignals {
    pub mouse: MouseSignals,
    pub keyboard: KeyboardSignals,
}
