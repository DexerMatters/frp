use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};
use take_mut::take;

use crate::signals::{apply, new_signal};

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
    pub fn unwrap_ref(&self) -> &T {
        match self {
            State::Change(current, _) => current,
            State::NoChange(current) => current,
        }
    }
}

pub type Effect<T> = fn(&T, &T);
pub type Guards<T> = Vec<Guard<T>>;
pub type Guard<T> = fn(&T, &T) -> bool;

#[derive(Debug)]
pub struct Signal<T> {
    pub state: Mutex<State<T>>,
    pub effect: Mutex<Effect<T>>,
    pub guard: Mutex<Guards<T>>,
}

pub type SignalArc<T> = Arc<Signal<T>>;

/**
 * Implement the Default trait for Signal<T> where T is a type that implements the Default trait.
 */
impl<T: Default> Default for Signal<T> {
    fn default() -> Self {
        Self {
            state: Mutex::new(State::NoChange(Default::default())),
            effect: Mutex::new(|_, _| {}),
            guard: Mutex::new(vec![|_, _| true]),
        }
    }
}

/**
 * Implement the Signalable trait for Signal<T> where T is a type that implements the Debug, Default, Send, and 'static traits.
 */
impl<T> Signal<T> {
    pub fn new(value: T) -> Self {
        Signal {
            state: Mutex::new(State::NoChange(value)),
            effect: Mutex::new(|_, _| {}),
            guard: Mutex::new(vec![|_, _| true]),
        }
    }

    pub fn replace(&self, value: T) {
        let mut state = self.state.lock();
        take(&mut *state, |state| match state {
            State::Change(old, _) => {
                self.effect.lock()(&value, &old);
                State::Change(value, old)
            }
            State::NoChange(old) => {
                self.effect.lock()(&value, &old);
                State::Change(value, old)
            }
        });
        MutexGuard::unlock_fair(state);
    }

    pub fn effect(&self) -> Effect<T> {
        *self.effect.lock()
    }

    pub fn join<U>(self: Arc<Self>, f: fn(T) -> U) -> SignalArc<U>
    where
        T: Clone,
        U: Default + Send + 'static,
        Arc<Self>: Send + 'static,
    {
        apply(new_signal(f), self)
    }

    pub fn apply<In, Out>(self: Arc<Self>, another: SignalArc<In>) -> SignalArc<Out>
    where
        SignalArc<In>: Send + 'static,
        Out: Send + 'static,
        In: Clone,
        T: Fn(In) -> Out + Send + Sync + Clone + 'static,
    {
        apply(self, another)
    }

    pub fn map<U>(self: Arc<Self>, f: fn(T) -> U) -> Arc<Signal<U>>
    where
        T: Clone + Send + 'static,
        U: Clone + Send + 'static,
    {
        apply(new_signal(f), self)
    }

    pub fn id(self: Arc<Self>) -> Arc<Signal<T>>
    where
        T: Clone + Default + Send + 'static,
    {
        apply(new_signal(|x: T| x.clone()), self)
    }

    pub fn with_effect(self: Arc<Self>, f: Effect<T>) -> Arc<Self> {
        *self.effect.lock() = f;
        self
    }

    pub fn with_guard(self: Arc<Self>, f: Guard<T>) -> Arc<Self> {
        self.guard.lock().push(f);
        self
    }

    pub fn with_diff_guard(self: Arc<Self>) -> Arc<Self>
    where
        T: PartialEq,
    {
        self.guard.lock().push(|x, y| x != y);
        self
    }

    // pub fn with<U>(self: Arc<Self>, another: Arc<Signal<U>>) -> Arc<Signal<(T, U)>>
    // where
    //     T: Clone + Default + Send + 'static,
    //     U: Clone + Default + Send + 'static,
    //     Arc<Self>: Send + 'static,
    // {
    //     prod::<T, U>(self, another)
    // }
}
