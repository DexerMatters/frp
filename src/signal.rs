use std::{fmt::Debug, mem, time::Duration};

use futures::lock::Mutex;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub enum State<T> {
    Change(T /* current */, T /* old */),
    NoChange(T),

    _Intermediate,
}

impl<T> State<T> {
    pub fn unwrap(self) -> T {
        match self {
            State::Change(current, _) => current,
            State::NoChange(current) => current,
            State::_Intermediate => unreachable!(),
        }
    }
}

pub type Effect<T> = fn(&T, &T);
pub type SignalLock<T> = Mutex<Signal<T>>;

#[derive(Debug)]
pub struct Signal<T> {
    state: State<T>,
    effect: Effect<T>,
}

impl<T: Default> Default for Signal<T> {
    fn default() -> Self {
        Self {
            state: State::NoChange(Default::default()),
            effect: |_, _| (),
        }
    }
}

impl<T: Debug> Signal<T> {
    pub fn new(value: T) -> Self {
        Signal {
            state: State::NoChange(value),
            effect: |_, _| (),
        }
    }

    pub fn effect(value: T, f: Effect<T>) -> Self {
        Signal {
            state: State::NoChange(value),
            effect: f,
        }
    }

    pub async fn change(&mut self, value: T) {
        let tmp = mem::replace(&mut self.state, State::_Intermediate);
        self.state = match tmp {
            State::Change(current, _) => {
                (self.effect)(&value, &current);
                State::Change(value, current)
            }
            State::NoChange(current) => {
                (self.effect)(&value, &current);
                State::Change(value, current)
            }
            State::_Intermediate => unreachable!(),
        };
    }

    pub async fn bind<'a, S: Debug>(
        this: &Mutex<Self>,
        right: &'a mut Signal<S>,
        f: fn(&T) -> S,
    ) -> &'a mut Signal<S> {
        dbg!("bind");
        loop {
            let mut this = this.lock().await;
            dbg!("State: {:?}", &this.state);
            sleep(Duration::from_millis(200)).await;
            match &this.state {
                State::Change(current, _) => {
                    dbg!("State: {:?}", &this.state);
                    right.change(f(current)).await;
                    let tmp = mem::replace(&mut this.state, State::_Intermediate);
                    this.state = State::NoChange(tmp.unwrap());
                    return right;
                }
                State::NoChange(_) => {
                    continue;
                }
                State::_Intermediate => unreachable!(),
            }
        }
    }
}
