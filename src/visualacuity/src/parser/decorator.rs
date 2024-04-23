use core::fmt::{Debug, Display, Formatter};

#[derive(PartialEq, Debug, Clone)]
pub enum DataQuality {
    Exact,
    Convertible,
    Unrecognized,
}

pub trait TInput: PartialEq + Debug + Clone {}
impl<T> TInput for T where T: PartialEq + Debug + Clone {}


#[derive(PartialEq, Debug, Clone)]
pub(crate) struct Content<'input, T: TInput> {
    pub(crate) content: T,
    pub(crate) input: &'input str,
    pub(crate) left: usize,
    pub(crate) right: usize,
    pub(crate) dq: DataQuality,
}

impl<'input, T: TInput> Content<'input, T> {
    pub(crate) fn map<U: TInput, M: Fn(&T) -> U>(&self, mapper: M) -> Content<'input, U> {
        let Content { input, left, right, dq, .. } = self.clone();
        Content { content: mapper(&self.content), input, left, right, dq }
    }

    pub(crate) fn input_string(&self) -> String {
        self.input[self.left..self.right].to_string()
    }
}

impl<'input, T: TInput> PartialEq<T> for Content<'input, T> {
    fn eq(&self, other: &T) -> bool {
        self.content == *other
    }
}

impl<'input, T: TInput> Display for Content<'input, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.input[self.left..self.right])
    }
}