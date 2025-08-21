use candid::Nat;
use ic_cdk_timers::TimerId;
use std::cell::RefCell;
use std::time::Duration;

use crate::signer::{top_up_cycles_ledger, TopUpCyclesLedgerRequest};

thread_local! {
    static TIMER_ID: RefCell<Option<TimerId>> = RefCell::new(None);
}

fn set_timer_interval(interval: Duration, func: impl FnMut() + 'static) -> TimerId {
    ic_cdk_timers::set_timer_interval(interval, func)
}

pub fn start_timer(interval: u64) {
    let timer_id = set_timer_interval(Duration::from_secs(interval), || {
        ic_cdk::spawn(async {
            top_up_cycles_ledger(TopUpCyclesLedgerRequest {
                threshold: Some(Nat::from(2_000_000_000_000u128)),
                percentage: None,
            })
            .await;
        });
    });

    TIMER_ID.with(|cell| {
        cell.replace(Some(timer_id));
    });
}

pub fn stop_timer() {
    TIMER_ID.with(|timer_id| {
        if let Some(timer_id) = timer_id.borrow_mut().take() {
            ic_cdk_timers::clear_timer(timer_id);
        }
    });
}
