use std::{mem::replace, sync::Arc, thread::sleep, time::Duration};
use tokio::spawn;

use impl_variadics::impl_variadics;

use crate::signal::{Signal, SignalRef};
use parking_lot::MutexGuard;

macro_rules! unlock_all {
    ($($lock:expr),*) => {
        $(MutexGuard::unlock_fair($lock);)*
    };
    () => {};
}

impl_variadics! {
    ..8 "T*" "b*" "self_lock_*" "state_lock_*" "value_lock_*" "effect_lock_*" => {
        impl<#(#T0),*> Signal<(#(SignalRef<#T0>,)*)>
        {
            pub fn join(self: Arc<Self>) -> SignalRef<(#(#T0,)*)>
            where
                #(#T0: 'static + Clone + Send + Sync,)*
            {
                self.join_(|#(#b0,)*| #(#b0&&)*true )
            }

            pub fn join_any(self: Arc<Self>) -> SignalRef<(#(#T0,)*)>
            where
                #(#T0: 'static + Clone + Send + Sync,)*
            {
                self.join_(|#(#b0,)*| #(#b0||)*false )
            }

            fn join_(self: Arc<Self>, pred: fn(#(bool,)*) -> bool) -> SignalRef<(#(#T0,)*)>
            where
                #(#T0: 'static + Clone + Send + Sync,)*
            {

                let self_value_lock = self.value.lock();
                #(let #self_lock_0 = self_value_lock.0.#index.value.lock();)*
                let new = Signal::new((#(#self_lock_0.0.clone(),)*));

                #(MutexGuard::unlock_fair(#self_lock_0);)*
                MutexGuard::unlock_fair(self_value_lock);
                let ret = new.clone();

                spawn(async move {
                    loop {
                        let new_effect_lock = new.effects.lock();
                        let mut new_value_lock = new.value.lock();
                        let mut new_state_lock = new.state.lock();
                        let self_value_lock = self.value.lock();
                        #(
                            let mut #state_lock_0 = self_value_lock.0.#index.state.lock();
                            let #value_lock_0 = self_value_lock.0.#index.value.lock();
                        )*

                        if pred(#(*#state_lock_0,)*) {
                            new_value_lock.1 = Some(replace(&mut new_value_lock.0, (#(#value_lock_0.0.clone(),)*)));
                            *new_state_lock = new_effect_lock.iter().fold(true, |head, effect| {
                                head && effect(&new_value_lock.0, &new_value_lock.1)
                            });
                            #(*#state_lock_0 = false;)*
                        }

                        unlock_all!(
                            #(#value_lock_0,)*
                            #(#state_lock_0,)*
                            new_state_lock,
                            new_value_lock,
                            new_effect_lock,
                            self_value_lock);

                        sleep(Duration::from_millis(10));
                    }
                });
                ret
            }
        }
    };
}
