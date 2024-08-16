use itertools::ExactlyOneError;
use std::fmt::{Debug, Display, Formatter};
use std::num::{ParseFloatError, ParseIntError};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VisualAcuityError {
    GenericError,
    ParseError(String),
    UnknownError(String),
    Unreachable,
    NotImplemented,
    LogMarInvalidSnellenRow(String),
    LogMarInvalidPlusLetters(String),
    DistanceConversionError,
    NoSnellenEquivalent(String),
    PlusLettersNotAllowed,
    NoValue,
    MultipleValues(String),
    VisitMetaError,
    MultipleErrors(Vec<VisualAcuityError>),
    ExtractNumbersError(String),
    ChartNotFound(String),
    ChartRowNotFound(String),
}

impl<L, T, E> From<lalrpop_util::ParseError<L, T, E>> for VisualAcuityError
where
    L: Debug,
    T: Debug,
    E: Debug,
{
    fn from(value: lalrpop_util::ParseError<L, T, E>) -> Self {
        VisualAcuityError::ParseError(format!("{value:?}"))
    }
}

impl From<ParseIntError> for VisualAcuityError {
    fn from(value: ParseIntError) -> Self {
        VisualAcuityError::ParseError(format!("{value:?}"))
    }
}

impl From<ParseFloatError> for VisualAcuityError {
    fn from(value: ParseFloatError) -> Self {
        VisualAcuityError::ParseError(format!("{value:?}"))
    }
}

impl<T> From<VisualAcuityError> for lalrpop_util::ParseError<usize, T, &str> {
    fn from(_: VisualAcuityError) -> Self {
        Self::User {
            error: "Parse error!",
        }
    }
}

impl<T: Clone + Into<VisualAcuityError>> From<&T> for VisualAcuityError {
    fn from(value: &T) -> Self {
        value.clone().into()
    }
}

impl<I: Iterator<Item = T>, T: ToString> From<ExactlyOneError<I>> for VisualAcuityError {
    fn from(value: ExactlyOneError<I>) -> Self {
        let mut it = value.into_iter();
        match it.next() {
            Some(item) => {
                let formatted = [item]
                    .into_iter()
                    .chain(it)
                    .map(|it| it.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                VisualAcuityError::MultipleValues(format!("[{formatted}]"))
            }
            None => VisualAcuityError::NoValue,
        }
    }
}

impl std::error::Error for VisualAcuityError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            VisualAcuityError::ParseError(_) => None,
            _ => None,
        }
    }
}

pub type VisualAcuityResult<T> = Result<T, VisualAcuityError>;

impl Display for VisualAcuityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VisualAcuityError::ParseError(e) => write!(f, "{}", e),
            _ => write!(f, "{self:?}"),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum OptionResult<T> {
    None,
    Some(T),
    Err(VisualAcuityError),
}

impl<T> OptionResult<T> {
    pub fn map<M, F: Fn(T) -> M>(self, f: F) -> OptionResult<M> {
        self.then(|v| OptionResult::Some(f(v)))
    }

    pub fn map_err<F: Fn(VisualAcuityError) -> VisualAcuityError>(self, f: F) -> Self {
        match self {
            Self::Err(e) => Self::Err(f(e)),
            _ => self,
        }
    }

    pub fn then<M, F: Fn(T) -> R, R: Into<OptionResult<M>>>(self, f: F) -> OptionResult<M> {
        match self {
            Self::Some(v) => f(v).into(),
            Self::None => OptionResult::None,
            Self::Err(e) => OptionResult::Err(e),
        }
    }
}

impl<T> Default for OptionResult<T> {
    fn default() -> Self {
        Self::None
    }
}

impl<T, V: Into<T>> From<Option<V>> for OptionResult<T> {
    fn from(value: Option<V>) -> Self {
        match value {
            None => Self::None,
            Some(v) => Self::Some(v.into()),
        }
    }
}

impl<T, E: Into<VisualAcuityError>> From<Result<T, E>> for OptionResult<T> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(v) => Self::Some(v),
            Err(e) => Self::Err(e.into()),
        }
    }
}

impl<T, E: Into<VisualAcuityError>> From<Result<Option<T>, E>> for OptionResult<T> {
    fn from(value: Result<Option<T>, E>) -> Self {
        value.map(|v| v).into()
    }
}
