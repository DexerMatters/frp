use std::{sync::Arc, thread::sleep, time::Duration};

use parking_lot::MutexGuard;
use take_mut::take;

use crate::signal::{Guards, Signal, SignalArc, State};

pub fn ref_apply<In, Out, F>(f: SignalArc<F>, a: SignalArc<In>) -> SignalArc<Out>
where
    SignalArc<In>: Send + 'static,
    Out: Send + 'static,
    F: Fn(&In) -> Out + Send + Sync + Clone + 'static,
{
    let func = { f.state.lock().unwrap_ref().clone() };
    let new = { new_signal(func(a.state.lock().unwrap_ref())) };
    let out = new.clone();
    tokio::spawn(async move {
        loop {
            let mut state = a.state.lock();
            take(&mut *state, |state| match state {
                State::Change(current, old) => {
                    if eval_guards(&*a.guard.lock(), &current, &old) {
                        new.replace(func(&current));
                    }
                    State::NoChange(current)
                }
                State::NoChange(current) => State::NoChange(current),
            });
            MutexGuard::unlock_fair(state);
            sleep(Duration::from_millis(10));
        }
    });
    out
}

pub fn apply<In, Out, F>(f: SignalArc<F>, a: SignalArc<In>) -> SignalArc<Out>
where
    SignalArc<In>: Send + 'static,
    Out: Send + 'static,
    In: Clone,
    F: Fn(In) -> Out + Send + Sync + Clone + 'static,
{
    let func = { f.state.lock().unwrap_ref().clone() };
    let new = { new_signal(func(a.state.lock().unwrap_ref().clone())) };
    let out = new.clone();

    tokio::spawn(async move {
        loop {
            let mut state = a.state.lock();
            let mut func_state = f.state.lock();

            take(&mut *func_state, |func_state| match func_state {
                State::Change(func, _) => {
                    take(&mut *state, |state| match state {
                        State::Change(current, old) => {
                            if eval_guards(&*a.guard.lock(), &current, &old) {
                                new.replace(func(current.clone()));
                            }
                            State::NoChange(current)
                        }
                        State::NoChange(current) => {
                            if eval_guards(&*a.guard.lock(), &current, &current) {
                                new.replace(func(current.clone()));
                            }
                            State::NoChange(current)
                        }
                    });
                    State::NoChange(func.clone())
                }
                State::NoChange(func) => {
                    take(&mut *state, |state| match state {
                        State::Change(current, old) => {
                            if eval_guards(&*a.guard.lock(), &current, &old) {
                                new.replace(func(current.clone()));
                            }
                            State::NoChange(current)
                        }
                        State::NoChange(current) => State::NoChange(current),
                    });
                    State::NoChange(func)
                }
            });
            MutexGuard::unlock_fair(func_state);
            MutexGuard::unlock_fair(state);
            sleep(Duration::from_millis(10));
        }
    });
    out
}

// pub fn prod<A, B>(a: SignalArc<A>, b: SignalArc<B>) -> SignalArc<(A, B)>
// where
//     SignalArc<A>: Send + 'static,
//     SignalArc<B>: Send + 'static,
//     A: Clone + Send + 'static,
//     B: Clone + Send + 'static,
// {
//     // Create a new signal that combines the two signals
//     let lock_a = a.state.lock();
//     let lock_b = b.state.lock();
//     let inital_a = (*lock_a).unwrap_ref().clone();
//     let inital_b = (*lock_b).unwrap_ref().clone();
//     let new = new_signal((inital_a, inital_b));
//     let out = new.clone();
//     MutexGuard::unlock_fair(lock_a);
//     MutexGuard::unlock_fair(lock_b);
//     tokio::spawn(async move {
//         loop {
//             let mut state = a.state.lock();
//             let mut another_state = b.state.lock();
//             take(&mut *state, |state| match state {
//                 State::Change(current, _) => {
//                     take(&mut *another_state, |another_state| match another_state {
//                         State::Change(another_current, _) => {
//                             if a.guard.lock()(&current) && b.guard.lock()(&another_current) {
//                                 new.replace((current.clone(), another_current.clone()));
//                             }
//                             State::NoChange(another_current)
//                         }
//                         State::NoChange(another_current) => {
//                             if a.guard.lock()(&current) && b.guard.lock()(&another_current) {
//                                 new.replace((current.clone(), another_current.clone()));
//                             }
//                             State::NoChange(another_current)
//                         }
//                     });
//                     State::NoChange(current)
//                 }
//                 State::NoChange(current) => {
//                     take(&mut *another_state, |another_state| match another_state {
//                         State::Change(another_current, _) => {
//                             if a.guard.lock()(&current) && b.guard.lock()(&another_current) {
//                                 new.replace((current.clone(), another_current.clone()));
//                             }
//                             State::NoChange(another_current)
//                         }
//                         State::NoChange(another_current) => State::NoChange(another_current),
//                     });
//                     State::NoChange(current)
//                 }
//             });
//             MutexGuard::unlock_fair(state);
//             MutexGuard::unlock_fair(another_state);

//             sleep(Duration::from_millis(10));
//         }
//     });
//     out
// }

pub fn new_signal<T: Send + 'static>(default: T) -> SignalArc<T> {
    Arc::new(Signal::new(default))
}

pub fn pure<T: Send + 'static>(default: T) -> SignalArc<T> {
    Arc::new(Signal::new(default))
}

fn eval_guards<T>(guards: &Guards<T>, current: &T, old: &T) -> bool {
    guards.iter().all(|guard| guard(current, old))
}
