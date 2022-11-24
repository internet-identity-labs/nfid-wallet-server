use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;

use candid::CandidType;
use serde::Deserialize;

use crate::notification_service::NotificationClass::Transaction;

// pub type Users = HashMap<String, User>;
thread_local! {
    static NOTIFICATIONS: RefCell<HashMap<u64, Vec<Notification>>> = RefCell::new(Default::default());
}


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Notification {
    pub id: u16,
    pub user_id: u64,
    pub class: NotificationClass,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum NotificationClass {
    #[serde(rename = "transaction")]
    Transaction(TransactionNotification)
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct TransactionNotification {
    pub transaction_id: u64,
}


pub fn register_notification(transaction_id: u64, user_ids: Vec<u64>) {
    let tn = TransactionNotification {
        transaction_id
    };

    NOTIFICATIONS.with(|notifications_rc| {
        for user_id in user_ids {
            match notifications_rc.borrow_mut().get_mut(&user_id)
            {
                None => {
                    let nt = Notification {
                        id: 1,
                        user_id,
                        class: (Transaction(tn.clone())),
                    };
                    notifications_rc.borrow_mut().insert(user_id, vec![nt]);
                }
                Some(user_norifications) => {
                    let id = user_norifications.len() + 1;
                    let nt = Notification {
                        id: id as u16,
                        user_id,
                        class: (Transaction(tn.clone())),
                    };
                    user_norifications.push(nt);
                }
            }
        }
    });
}



