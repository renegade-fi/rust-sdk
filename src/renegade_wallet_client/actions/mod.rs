//! Actions to update a Renegade wallet

pub mod admin_assign_order_to_pool;
pub mod admin_create_matching_pool;
pub mod admin_get_account_orders;
pub mod admin_get_open_orders;
pub mod admin_get_order;
pub mod admin_is_task_queue_paused;
pub mod admin_place_order_in_pool;
pub mod cancel_order;
pub mod create_account;
pub mod deposit;
pub mod get_account_seeds;
pub mod get_balance_by_mint;
pub mod get_balances;
pub mod get_order;
pub mod get_orders;
pub mod get_task;
pub mod get_tasks;
pub mod place_order;
pub mod sync_account;
pub mod update_order;
pub mod withdraw;

// -----------
// | Helpers |
// -----------

/// Constructs an HTTP path by replacing URL parameters with given values
macro_rules! construct_http_path {
    ($base_url:expr $(, $param:expr => $value:expr)*) => {{
        let mut url = $base_url.to_string();
        $(
            let placeholder = format!(":{}", $param);
            url = url.replace(&placeholder, &$value.to_string());
        )*
        url
    }};
}
pub(crate) use construct_http_path;
