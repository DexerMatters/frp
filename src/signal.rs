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
pub type Guard<T> = fn(&T) -> bool;

#[derive(Debug)]
pub struct Signal<T> {
    state: Mutex<State<T>>,
    effect: Mutex<Effect<T>>,
    guard: Mutex<Guard<T>>,
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
            guard: Mutex::new(|_| true),
        }
    }
}

/**
 * Implement the Signalable trait for Signal<T> where T is a type that implements the Debug, Default, Send, and 'static traits.
 */
impl<T: Default> Signal<T> {
    pub fn new(value: T) -> Self {
        Signal {
            state: Mutex::new(State::NoChange(value)),
            effect: Mutex::new(|_, _| true),
            guard: Mutex::new(|_| true),
        }
    }

    pub fn from_effect(value: T, f: Effect<T>) -> Self {
        Signal {
            state: Mutex::new(State::NoChange(value)),
            effect: Mutex::new(f),
            guard: Mutex::new(|_| true),
        }
    }

    pub fn replace(&self, value: T) -> bool {
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

    pub fn effect(&self) -> Effect<T> {
        *self.effect.lock()
    }

    pub fn map<U>(self: Arc<Self>, f: fn(&T) -> U) -> Arc<Signal<U>>
    where
        U: Default + Send + 'static,
        Arc<Self>: Send + 'static,
    {
        let signal = Arc::new(Signal::default());
        let ret = signal.clone();
        let this = self.clone();
        tokio::spawn(async move {
            loop {
                let mut will_break = false;
                let mut state = this.state.lock();
                take(&mut *state, |state| match state {
                    State::Change(current, _) => {
                        if this.guard.lock()(&current) {
                            will_break = !signal.replace(f(&current));
                        }
                        State::NoChange(current)
                    }
                    State::NoChange(current) => State::NoChange(current),
                });
                if will_break {
                    break;
                }
                MutexGuard::unlock_fair(state);
                sleep(Duration::from_millis(10));
            }
        });
        ret
    }

    pub fn forward(self: Arc<Self>) -> Arc<Signal<T>>
    where
        T: Clone + Default + Send + 'static,
    {
        self.map(|x| x.clone())
    }

    pub fn with_effect(self: Arc<Self>, f: Effect<T>) -> Arc<Self> {
        *self.effect.lock() = f;
        self
    }

    pub fn with_guard(self: Arc<Self>, f: Guard<T>) -> Arc<Self> {
        *self.guard.lock() = f;
        self
    }

    pub fn with<U>(self: Arc<Self>, another: Arc<Signal<U>>) -> Arc<Signal<(T, U)>>
    where
        T: Clone + Default + Send + 'static,
        U: Clone + Default + Send + 'static,
        Arc<Self>: Send + 'static,
    {
        let signal = Arc::new(Signal::default());
        let ret = signal.clone();
        let this = self.clone();
        let that = another.clone();
        tokio::spawn(async move {
            loop {
                let mut state = this.state.lock();
                let mut another_state = that.state.lock();
                let mut will_break = false;
                take(&mut *state, |state| match state {
                    State::Change(current, _) => {
                        take(&mut *another_state, |another_state| match another_state {
                            State::Change(another_current, _) => {
                                if this.guard.lock()(&current)
                                    && that.guard.lock()(&another_current)
                                {
                                    will_break =
                                        !signal.replace((current.clone(), another_current.clone()));
                                }
                                State::NoChange(another_current)
                            }
                            State::NoChange(another_current) => State::NoChange(another_current),
                        });
                        State::NoChange(current)
                    }
                    State::NoChange(current) => {
                        take(&mut *another_state, |another_state| match another_state {
                            State::Change(another_current, _) => {
                                if this.guard.lock()(&current)
                                    && that.guard.lock()(&another_current)
                                {
                                    will_break =
                                        !signal.replace((current.clone(), another_current.clone()));
                                }
                                State::NoChange(another_current)
                            }
                            State::NoChange(another_current) => State::NoChange(another_current),
                        });
                        State::NoChange(current)
                    }
                });
                if will_break {
                    break;
                }
                MutexGuard::unlock_fair(state);
                MutexGuard::unlock_fair(another_state);

                sleep(Duration::from_millis(10));
            }
        });
        ret
    }
}
