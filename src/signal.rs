use std::{fmt::Debug, mem, time::Duration};

use parking_lot::{Mutex, MutexGuard};
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
    pub fn new(value: T) -> SignalLock<T> {
        Mutex::new(Signal {
            state: State::NoChange(value),
            effect: |_, _| (),
        })
    }

    pub fn effect(value: T, f: Effect<T>) -> SignalLock<T> {
        Mutex::new(Signal {
            state: State::NoChange(value),
            effect: f,
        })
    }

    pub async fn change(this: &Mutex<Self>, value: T) {
        let mut this = this.lock();
        let tmp = mem::replace(&mut this.state, State::_Intermediate);
        this.state = match tmp {
            State::Change(current, _) => {
                (this.effect)(&value, &current);
                State::Change(value, current)
            }
            State::NoChange(current) => {
                (this.effect)(&value, &current);
                State::Change(value, current)
            }
            State::_Intermediate => unreachable!(),
        };
        MutexGuard::unlock_fair(this);
    }

    pub async fn bind<'a, S: Debug>(
        this: &SignalLock<T>,
        right: &'a mut SignalLock<S>,
        f: fn(&T) -> S,
    ) -> &'a mut SignalLock<S> {
        dbg!("bind");
        loop {
            let mut this = this.lock();
            sleep(Duration::from_millis(200)).await;
            match &this.state {
                State::Change(current, _) => {
                    println!("State: {:?}", &this.state);
                    Signal::change(right, f(current)).await;
                    let tmp = mem::replace(&mut this.state, State::_Intermediate);
                    this.state = State::NoChange(tmp.unwrap());
                    continue;
                }
                State::NoChange(_) => {
                    continue;
                }
                State::_Intermediate => unreachable!(),
            }
        }
    }
}
