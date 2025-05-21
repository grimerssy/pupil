pub mod auth;

pub mod error;

pub mod email;
pub mod name;
pub mod password;

mod prelude {
    pub(super) use super::define;

    pub use super::{email::*, name::*, password::*};
}

macro_rules! define {
    () => {};
    (operation $action:ident = $($async:ident)? ($input:ty) -> { $success:ty, $error:ty };) => {
        ::paste::paste! {
            pub trait $action {
                $($async)? fn [<$action:snake>](
                    &self,
                    [<$input:snake>]: $input,
                ) -> $crate::domain::error::DomainResult<$success, $error>;
            }
        }
    };
    (record $record:ident = ($( $field:ident ),* $(,)? );) => {
        ::paste::paste! {
            #[derive(Debug, Clone)]
            pub struct $record {
                $(
                    pub $field: [<$field:camel>],
                )*
            }
        }
    };
    (error $error:ident = { $( $variant:ident ),*  $(,)? };) => {
        ::paste::paste! {
            #[derive(Debug)]
            pub enum $error {
                $( $variant, )*
            }

            impl $crate::app::error::ContextualError for $error {
                fn error_context(self) -> $crate::app::error::ErrorContext {
                    match self {
                        $(
                            Self::$variant => $crate::app::error::ErrorContext::new(
                                stringify!([<$variant:snake:upper>])
                            )
                        )*
                    }
                }
            }
        }
    };
    (error $error:ident = { $( ($variant:ident) ),*  $(,)? };) => {
        #[derive(Debug)]
        pub enum $error {
            $( $variant($variant), )*
        }

        $(
            impl From<$variant> for $error {
                fn from(value: $variant) -> Self {
                    Self::$variant(value)
                }
            }
        )*

        impl $crate::app::error::ContextualError for $error {
            fn error_context(self) -> $crate::app::error::ErrorContext {
                match self {
                    $(
                        Self::$variant(error) => error.error_context(),
                    )*
                }
            }
        }
    };
    (operation $action:ident = $($async:ident)? ($input:ty) -> { $ok:ty, $err:ty }; $( $rest:tt )*) => {
        $crate::domain::define!(operation $action = $( $async )? ($input) -> { $ok, $err };);
        $crate::domain::define!($( $rest )*);
    };
    (record $record:ident = ( $( $field:ident ),* $(,)? ); $( $rest:tt )*) => {
        $crate::domain::define!(record $record = ( $( $field, )* ););
        $crate::domain::define!($( $rest )*);
    };
    (error $error:ident = { $( $variant:ident ),*  $(,)? }; $( $rest:tt )*) => {
        $crate::domain::define!(error $error = { $( $variant, )* };);
        $crate::domain::define!($( $rest )*);
    };
    (error $error:ident = { $( ($variant:ident) ),*  $(,)? }; $( $rest:tt )*) => {
        $crate::domain::define!(error $error = { $( ($variant), )* };);
        $crate::domain::define!($( $rest )*);
    };
}

use define;
