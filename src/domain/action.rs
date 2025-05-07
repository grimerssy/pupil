use async_fn_traits::AsyncFn1;

pub trait AsyncAction<I, O>: AsyncFn1<I, OutputFuture: Send, Output = O> {}

impl<I, O, F, Fut> AsyncAction<I, O> for F
where
    F: Fn(I) -> Fut,
    Fut: Future<Output = O> + Send,
{
}

macro_rules! action {
    ($visibility:vis $action:ident = ($input:ty) -> $output:ty) => {
        ::trait_set::trait_set! {
            $visibility trait $action = Fn($input) -> $output;
        }
    };
    ($visibility:vis $action:ident = async ($input:ty) -> $output:ty) => {
        ::trait_set::trait_set! {
            $visibility trait $action = $crate::domain::action::AsyncAction<$input, $output>;
        }
    };
    ( $( $visibility:vis $action:ident = $($async:ident)? ($input:ty) -> $output:ty; )* ) => {
        $( $crate::domain::auth::action!($visibility $action = $($async)? ($input) -> $output); )*
    };
}

pub(crate) use action;
