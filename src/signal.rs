use std::{fmt::Debug, sync::Arc, thread::sleep, time::Duration};

use parking_lot::{Mutex, MutexGuard};
use take_mut::take;

#[derive(Debug, Clone)]
pub enum State<T> {
    Change(T /* current */, T /* old */),
    NoChange(T),
}

impl<T> State<T> {
    pub fn unwrap(self) -> T {
        match self {
            State::Change(current, _) => current,
            State::NoChange(current) => current,
        }
    }
    pub fn unwrap_old(self) -> T {
        match self {
            State::Change(_, old) => old,
            State::NoChange(old) => old,
        }
    }
}

pub type Effect<T> = fn(&T, &T) -> bool;

#[derive(Debug)]
pub struct Signal<T> {
    state: Mutex<State<T>>,
    effect: Mutex<Effect<T>>,
}

pub type SignalArc<T> = Arc<Signal<T>>;

/**
 * Implement the Default trait for Signal<T> where T is a type that implements the Default trait.
 */
impl<T: Default> Default for Signal<T> {
    fn default() -> Self {
        Self {
            state: Mutex::new(State::NoChange(Default::default())),
            effect: Mutex::new(|_, _| true),
        }
    }
}

/**
 * Implement the Signalable trait for Signal<T> where T is a type that implements the Debug, Default, Send, and 'static traits.
 */
impl<T: Debug + Default + Send + 'static> Signalable<T> for Signal<T> {
    fn new(value: T) -> Self {
        Signal {
            state: Mutex::new(State::NoChange(value)),
            effect: Mutex::new(|_, _| true),
        }
    }

    fn from_effect(value: T, f: Effect<T>) -> Self {
        Signal {
            state: Mutex::new(State::NoChange(value)),
            effect: Mutex::new(f),
        }
    }

    fn replace(&self, value: T) -> bool {
        let mut state = self.state.lock();
        let mut re = true;
        take(&mut *state, |state| match state {
            State::Change(old, _) => {
                re = self.effect.lock()(&value, &old);
                State::Change(value, old)
            }
            State::NoChange(old) => {
                re = self.effect.lock()(&value, &old);
                State::Change(value, old)
            }
        });
        MutexGuard::unlock_fair(state);
        re
    }

    fn effect(&self) -> Effect<T> {
        *self.effect.lock()
    }

    fn map<S, U>(self: Arc<Self>, f: fn(&T) -> U) -> Arc<S>
    where
        U: Default + 'static,
        S: Signalable<U> + Send + Sync + 'static,
    {
        let signal = Arc::new(S::default());
        let ret = signal.clone();
        let this = self.clone();
        tokio::spawn(async move {
            loop {
                let mut will_break = false;
                let mut state = this.state.lock();
                sleep(Duration::from_millis(10));
                take(&mut *state, |state| match state {
                    State::Change(current, _) => {
                        if !signal.replace(f(&current)) {
                            will_break = true;
                        };
                        State::NoChange(current)
                    }
                    State::NoChange(current) => State::NoChange(current),
                });
                if will_break {
                    break;
                }
                MutexGuard::unlock_fair(state);
            }
        });
        ret
    }

    fn with_effect(self: Arc<Self>, f: Effect<T>) -> Arc<Self> {
        *self.effect.lock() = f;
        self
    }
}

pub trait Signalable<T>: Default {
    fn new(value: T) -> Self;
    fn from_effect(value: T, f: Effect<T>) -> Self;
    fn replace(&self, value: T) -> bool;
    fn effect(&self) -> Effect<T>;
    fn with_effect(self: Arc<Self>, f: Effect<T>) -> Arc<Self>;
    fn map<S, U>(self: Arc<Self>, f: fn(&T) -> U) -> Arc<S>
    where
        U: Default + 'static,
        S: Signalable<U> + Send + Sync + 'static;
}
