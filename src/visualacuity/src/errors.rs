use std::fmt::{Debug, Display, Formatter};
use std::num::{ParseFloatError, ParseIntError};
use itertools::ExactlyOneError;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VisualAcuityError {
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
        Self::User { error: "Parse error!" }
    }
}

impl<T: Clone + Into<VisualAcuityError>> From<&T> for VisualAcuityError {
    fn from(value: &T) -> Self { value.clone().into() }
}

impl<I: Iterator<Item=T>, T: Debug> From<ExactlyOneError<I>> for VisualAcuityError {
    fn from(value: ExactlyOneError<I>) -> Self {
        let mut it = value.into_iter();
        match it.next() {
            Some(item) => {
                let items = [item].into_iter().chain(it).collect::<Vec<_>>();
                VisualAcuityError::MultipleValues(format!("{items:?}"))
            },
            None => VisualAcuityError::NoValue
        }
    }
}

impl std::error::Error for VisualAcuityError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            VisualAcuityError::ParseError(_) => None,
            _ => None
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
