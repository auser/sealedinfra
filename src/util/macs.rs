use std::fmt::{Display, Formatter};

use anyhow::Result;
use axum::http::StatusCode;

/// Returns early with an error. This macro is similar to the `bail!` macro which can be found in `anyhow`.
/// This macro is equivalent to `return Err(err!(...))`.
///
/// # Example
///
/// ```
/// # fn is_valid(input: &str) -> bool {
/// #     true
/// # }
/// #
/// # fn main() -> anyhow::Result<()> {
/// #     let input = "";
/// #
/// use crate::die;
///
/// if !is_valid("input") {
///     die!(BAD_REQUEST, "Received invalid input");
/// }
/// #
/// #     Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! die {
    ($($input:tt)*) => {
        return Err($crate::err!($($input)*).into())
    }
}

/// Constructs a new error with a status code or from an existing error.
/// This macro is similar to the `anyhow!` macro which can be found in `anyhow`.
///
/// # Example
///
/// ```
/// # fn process_input(input: &str) -> Result<()> {
/// #     Ok(())
/// # }
/// #
/// # fn main() -> Result<()> {
/// #     let input = "";
/// #
/// use crate::err;
///
/// process_input(input).map_err(|_| err!(BAD_REQUEST, "Received invalid input"))?;
/// #
/// #     Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! err {
    ($code:ident) => {
        $crate::error::WithStatusCode::new(actix_web::http::StatusCode::$code)
    };
    ($code:literal) => {{
        use anyhow::Context as _;

        $crate::error::WithStatusCode::try_new($code).context("Tried to die with invalid status code")?.into()
    }};
    ($code:ident, $message:literal) => {
        $crate::error::WithStatusCode {
            code: actix_web::http::StatusCode::$code,
            source: Some(anyhow::anyhow!($message)),
            display: true
        }
    };
    ($err:expr $(,)?) => ({
        $crate::error::WithStatusCode {
            code: actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            source: Some(anyhow::anyhow!($err)),
            display: false
        }
    });
    ($code:ident, $fmt:literal, $($arg:tt)*) => {
        $crate::error::WithStatusCode {
            code: actix_web::http::StatusCode::$code,
            source: Some(anyhow::anyhow!($fmt, $($arg)*)),
            display: true
        }
    };
}

#[derive(Debug)]
pub(crate) struct WithStatusCode {
    pub(crate) code: StatusCode,
    pub(crate) source: Option<anyhow::Error>,
    pub(crate) display: bool, // Whenever cause() should be shown to the user
}

impl Display for WithStatusCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            Some(source) if self.display => write!(f, "{}", source),
            _ => write!(
                f,
                "{} {}",
                self.code.as_str(),
                self.code.canonical_reason().unwrap_or_default()
            ),
        }
    }
}

impl WithStatusCode {
    pub(crate) fn new(code: StatusCode) -> WithStatusCode {
        WithStatusCode {
            code,
            source: None,
            display: false,
        }
    }

    pub(crate) fn try_new(code: u16) -> Result<WithStatusCode> {
        Ok(WithStatusCode {
            code: StatusCode::from_u16(code)?,
            source: None,
            display: false,
        })
    }
}
