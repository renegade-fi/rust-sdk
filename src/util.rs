//! Utility functions for the renegade-sdk

/// Wrap an eyre result in a function
#[macro_export]
macro_rules! wrap_eyre {
    ($x:expr) => {
        $x.map_err(|err| eyre::eyre!(err.to_string()))
    };
}

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

// Export macros
pub(crate) use construct_http_path;
pub(crate) use wrap_eyre;
