use std::{
    fmt::{Debug, Display},
    time::Duration,
};

use parking_lot::{Mutex, MutexGuard};
use take_mut::take;
use tokio::time::sleep;

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
}

pub type Effect<T> = fn(&T, &T);

#[derive(Debug)]
pub struct Signal<T> {
    state: Mutex<State<T>>,
    effect: Effect<T>,
}

impl<T: Default> Default for Signal<T> {
    fn default() -> Self {
        Self {
            state: Mutex::new(State::NoChange(Default::default())),
            effect: |_, _| (),
        }
    }
}

impl<T: Debug> Signal<T> {
    pub fn new(value: T) -> Self {
        Signal {
            state: Mutex::new(State::NoChange(value)),
            effect: |_, _| (),
        }
    }

    pub fn effect(value: T, f: Effect<T>) -> Self {
        Signal {
            state: Mutex::new(State::NoChange(value)),
            effect: f,
        }
    }

    pub async fn change(&self, value: T) {
        let mut state = self.state.lock();
        take(&mut *state, |state| match state {
            State::Change(old, _) => {
                (self.effect)(&value, &old);
                State::Change(value, old)
            }
            State::NoChange(old) => {
                (self.effect)(&value, &old);
                State::Change(value, old)
            }
        });
        MutexGuard::unlock_fair(state);
    }

    pub async fn map_<'a, S: Debug>(&self, right: &'a Signal<S>, f: fn(&T) -> S) {
        dbg!("bind");
        loop {
            sleep(Duration::from_millis(10)).await;
            let mut state = self.state.lock();
            match &*state {
                State::Change(current, _) => {
                    right.change(f(&current)).await;
                    take(&mut *state, |state| match state {
                        State::Change(current, _) => State::NoChange(current),
                        State::NoChange(_) => unreachable!(),
                    });
                    MutexGuard::unlock_fair(state);
                    continue;
                }
                State::NoChange(_) => {
                    MutexGuard::unlock_fair(state);
                    continue;
                }
            }
        }
    }

    pub async fn map<S: Default + Debug>(&self, f: fn(&T) -> S) -> Signal<S> {
        println!("map");
        let right = Signal::<S>::default();
        self.map_(&right, f).await;
        right
    }
}

impl<T: Display + Debug> Signal<T> {
    pub async fn print(&self) {
        let mut print = Signal::effect("".to_string(), |new, _| {
            println!("{}", new);
        });
        self.map_(&mut print, |x| format!("{:?}", x)).await;
    }
}
