use std::{mem::replace, sync::Arc, thread::sleep, time::Duration};

use parking_lot::{Mutex, MutexGuard};
use tokio::spawn;

pub type SignalRef<T> = Arc<Signal<T>>;
pub type Effect<T> = fn(&T, &Option<T>) -> bool;

macro_rules! unlock_all {
    ($($lock:expr),*) => {
        $(MutexGuard::unlock_fair($lock);)*
    };
    () => {};
}

#[derive(Default)]
pub struct Signal<T> {
    pub(crate) value: Mutex<(T /* current */, Option<T> /* history */)>,
    pub(crate) state: Mutex<bool>, // true for updated and false for not updated
    pub(crate) effects: Mutex<Vec<Effect<T>>>,
}

impl<T> Signal<T> {
    pub fn new(value: T) -> SignalRef<T> {
        Arc::new(Self {
            value: Mutex::new((value, None)),
            state: Mutex::new(false),
            effects: Mutex::new(Vec::new()),
        })
    }

    pub fn effect(self: SignalRef<T>, effect: Effect<T>) -> SignalRef<T> {
        self.effects.lock().push(effect);
        self
    }

    pub fn strict(self: SignalRef<T>) -> SignalRef<T>
    where
        T: PartialEq,
    {
        self.effect(|s, t| match t {
            None => true,
            Some(history) => *history != *s,
        })
    }

    pub fn ignore(self: SignalRef<T>) -> SignalRef<T> {
        self.effect(|_, _| false)
    }

    pub async fn update(self: SignalRef<T>, value: T) {
        let mut state_lock = self.state.lock();
        let mut value_lock = self.value.lock();
        let effect_lock = self.effects.lock();
        value_lock.1 = Some(replace(&mut value_lock.0, value));
        if !*state_lock {
            *state_lock = effect_lock.iter().fold(true, |head, effect| {
                head && effect(&value_lock.0, &value_lock.1)
            });
        }
        unlock_all!(state_lock, value_lock, effect_lock);
    }

    pub fn apply<F, S>(self: SignalRef<T>, a: SignalRef<F>) -> SignalRef<S>
    where
        T: Fn(F) -> S + Clone + Send + Sync + 'static,
        S: Send + Sync + 'static,
        F: Clone + Send + Sync + 'static,
    {
        let lock = self.value.lock();
        let lock_ = a.value.lock();
        let new = Signal::new(lock.0.clone()(lock_.0.clone()));

        unlock_all!(lock, lock_);
        let ret = new.clone();
        spawn(async move {
            loop {
                let mut arg_state_lock = a.state.lock();
                let mut self_state_lock = self.state.lock();

                let mut new_value_lock = new.value.lock();
                let mut new_state_lock = new.state.lock();

                let new_effects_lock = new.effects.lock();
                let self_value_lock = self.value.lock();
                let arg_value_lock = a.value.lock();

                if *arg_state_lock || *self_state_lock {
                    let value = self_value_lock.0(arg_value_lock.0.clone());
                    new_value_lock.1 = Some(replace(&mut new_value_lock.0, value));
                    *new_state_lock = new_effects_lock.iter().fold(true, |head, effect| {
                        head && effect(&new_value_lock.0, &new_value_lock.1)
                    });

                    *arg_state_lock = false;
                    *self_state_lock = false;
                }
                unlock_all!(
                    arg_state_lock,
                    self_state_lock,
                    new_value_lock,
                    new_state_lock,
                    new_effects_lock,
                    self_value_lock,
                    arg_value_lock
                );
                sleep(Duration::from_millis(10));
            }
        });
        ret
    }
}
