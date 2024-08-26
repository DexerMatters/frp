use std::{mem::replace, sync::Arc, thread::sleep, time::Duration};

use parking_lot::{Mutex, MutexGuard};
use tokio::spawn;

pub type SignalRef<T> = Arc<Signal<T>>;
pub type Effect<T> = fn(Arc<Signal<T>>) -> bool;

#[derive(Default)]
pub struct Signal<T> {
    value: Mutex<(T /* current */, Option<T> /* history */)>,
    state: Mutex<bool>, // true for updated and false for not updated
    effects: Mutex<Vec<Effect<T>>>,
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
        self.effect(|s| s.has_changed())
    }

    pub fn unwrap(self: SignalRef<T>) -> T
    where
        T: Clone,
    {
        self.value.lock().0.clone()
    }

    pub fn update(self: SignalRef<T>, value: T) -> bool {
        self.value.lock().1 = Some(replace(&mut self.value.lock().0, value));
        replace(&mut *self.state.lock(), true)
    }

    pub fn has_changed(self: SignalRef<T>) -> bool
    where
        T: PartialEq,
    {
        match &self.clone().value.lock().1 {
            None => false,
            Some(history) => *history != self.clone().value.lock().0,
        }
    }

    pub fn apply<F, S>(self: SignalRef<T>, a: SignalRef<F>) -> SignalRef<S>
    where
        T: Fn(F) -> S + Clone + Send + Sync + 'static,
        S: Send + Sync + 'static,
        F: Clone + Send + Sync + 'static,
    {
        let func_lock = self.value.lock();
        let arg_lock = a.value.lock();
        let func = func_lock.0.clone();
        let value = func(arg_lock.0.clone());
        let new = Signal::new(value);
        let ret = new.clone();
        MutexGuard::unlock_fair(func_lock);
        MutexGuard::unlock_fair(arg_lock);
        spawn(async move {
            loop {
                let ps = *a.state.lock() && a.clone().run_effect();
                let pf = *self.state.lock() && self.clone().run_effect();
                let _ = ps && pf && {
                    let value = self.value.lock().0(a.value.lock().0.clone());
                    new.clone().update(value)
                };
                sleep(Duration::from_millis(20));
            }
        });
        ret
    }

    fn run_effect(self: SignalRef<T>) -> bool {
        *self.state.lock() = false;
        self.effects
            .lock()
            .iter()
            .fold(true, |head, effect| head && effect(self.clone()))
    }
}
