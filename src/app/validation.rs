pub(crate) use macros::*;

use educe::Educe;

use core::fmt;
use std::collections::HashMap;

use serde::Serialize;

use super::localization::LocalizedError;

#[derive(Debug)]
pub struct Validation<I> {
    state: ValidationFailure<I>,
}

#[derive(Debug)]
pub struct ValidationFailure<I> {
    pub input: I,
    pub errors: Vec<LocalizedError>,
}

#[derive(Educe, Clone, Default, Serialize)]
#[educe(Debug)]
pub struct ValidationErrors(
    #[educe(Debug(method(fmt_keys)))] HashMap<&'static str, Vec<LocalizedError>>,
);

impl<I> Validation<I> {
    pub fn new(input: I) -> Self {
        let state = ValidationFailure {
            input,
            errors: Vec::new(),
        };
        Self { state }
    }

    pub fn check_or_else(
        mut self,
        predicate: impl Fn(&I) -> bool,
        error: impl FnOnce() -> LocalizedError,
    ) -> Self {
        if !predicate(&self.state.input) {
            self.state.errors.push(error());
        }
        self
    }

    pub fn finish(self) -> Result<I, ValidationFailure<I>> {
        if self.state.errors.is_empty() {
            Ok(self.state.input)
        } else {
            Err(self.state)
        }
    }
}

impl ValidationErrors {
    pub fn add(&mut self, field: &'static str, mut errors: Vec<LocalizedError>) {
        self.0
            .entry(field)
            .and_modify(|e| e.append(&mut errors))
            .or_insert(errors);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

fn fmt_keys<K, V>(map: &HashMap<K, V>, f: &mut fmt::Formatter<'_>) -> fmt::Result
where
    K: fmt::Debug,
{
    fmt::Debug::fmt(&map.keys(), f)
}

mod macros {
    macro_rules! try_convert {
        ($src:ident $input:expr => $dest:ident {$($field:ident),* $(,)?}) => {{
            let mut errors = $crate::app::validation::ValidationErrors::default();
            $(
                let $field = $input
                    .$field
                    .try_into()
                    .map_err($crate::app::validation::ValidationFailure::from)
                    .map_err(|failure| {
                        let field = stringify!($field);
                        errors.add(field, failure.errors);
                        failure.input
                    });
            )*
            if errors.is_empty() {
                $(
                    let $field = $field.unwrap();
                )*
                Ok($dest { $( $field, )* })
            } else {
                Err(errors)
            }
        }};
    }

    pub(crate) use try_convert;
}
