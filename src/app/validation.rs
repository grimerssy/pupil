use educe::Educe;
pub(crate) use macros::*;

use core::fmt;
use std::collections::HashMap;

use serde::{ser::SerializeSeq, Serialize, Serializer};

#[derive(Debug)]
pub struct Validation<I> {
    state: ValidationFailure<I>,
}

#[derive(Debug, thiserror::Error)]
#[error("{errors}")]
pub struct ValidationFailure<I> {
    pub input: I,
    pub errors: ErrorList,
}

#[derive(Debug, thiserror::Error, Serialize)]
#[error("{errors}")]
pub struct ConversionFailure<I> {
    pub input: I,
    pub errors: ValidationErrors,
}

#[derive(Educe, Default, thiserror::Error, Serialize)]
#[educe(Debug)]
#[error("{self:?}")]
pub struct ValidationErrors(#[educe(Debug(method(fmt_keys)))] HashMap<&'static str, ErrorList>);

#[derive(Default, thiserror::Error)]
#[error("{self:?}")]
pub struct ErrorList(Vec<anyhow::Error>);

impl<I> Validation<I> {
    pub fn new(input: I) -> Self {
        let state = ValidationFailure {
            input,
            errors: ErrorList::default(),
        };
        Self { state }
    }

    pub fn check<T, E>(mut self, check: impl FnOnce(&I) -> Result<T, E>) -> Self
    where
        E: Into<anyhow::Error>,
    {
        if let Err(error) = check(&self.state.input) {
            self.state.errors.add(error.into());
        }
        self
    }

    pub fn check_or_else<E>(
        mut self,
        predicate: impl Fn(&I) -> bool,
        error: impl FnOnce() -> E,
    ) -> Self
    where
        E: Into<anyhow::Error>,
    {
        if !predicate(&self.state.input) {
            self.state.errors.add(error().into());
        }
        self
    }

    pub fn finish(self) -> Result<I, ValidationFailure<I>> {
        if self.state.errors.0.is_empty() {
            Ok(self.state.input)
        } else {
            Err(self.state)
        }
    }
}

impl ValidationErrors {
    pub fn add(&mut self, field: &'static str, mut errors: ErrorList) {
        self.0
            .entry(field)
            .and_modify(|e| e.append(&mut errors))
            .or_insert(errors);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl ErrorList {
    pub fn add(&mut self, error: anyhow::Error) {
        self.0.push(error)
    }

    fn append(&mut self, errors: &mut Self) {
        self.0.append(&mut errors.0)
    }
}

impl fmt::Debug for ErrorList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        for error in self.0.iter() {
            list.entry(&format_args!("{error}"));
        }
        list.finish()
    }
}

impl Serialize for ErrorList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for error in self.0.iter() {
            seq.serialize_element(&format_args!("{error}"))?;
        }
        seq.end()
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
                $(
                    let $field = match $field {
                        Ok(wrapper) => wrapper.into(),
                        Err(raw) => raw,
                    };
                )*
                let input = $src { $( $field, )* };
                Err($crate::app::validation::ConversionFailure{ input, errors })
            }
        }};
    }

    pub(crate) use try_convert;
}
