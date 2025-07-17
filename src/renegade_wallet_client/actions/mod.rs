//! Actions to update a Renegade wallet

pub mod create_wallet;
pub mod get_wallet;

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
