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
                    self,
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
    (error $error:ident = { $( $variant:ident = $msg:literal ),*  $(,)? };) => {
        #[derive(Debug, ::thiserror::Error)]
        pub enum $error {
            $(
                #[error($msg)]
                $variant,
            )*
        }
    };
    (error $error:ident = { $( $variant:ident ),*  $(,)? };) => {
        #[derive(Debug, ::thiserror::Error)]
        pub enum $error {
            $(
                #[error(transparent)]
                $variant(#[from] $variant),
            )*
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
    (error $error:ident = { $( $variant:ident = $msg:literal ),*  $(,)? }; $( $rest:tt )*) => {
        $crate::domain::define!(error $error = { $( $variant = $msg, )* };);
        $crate::domain::define!($( $rest )*);
    };
    (error $error:ident = { $( $variant:ident ),*  $(,)? }; $( $rest:tt )*) => {
        $crate::domain::define!(error $error = { $( $variant, )* };);
        $crate::domain::define!($( $rest )*);
    };
}

use define;
