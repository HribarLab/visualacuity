use crate::dataquality::DataQuality;
use crate::DataQuality::*;
use core::fmt::{Debug, Display, Formatter};

pub trait TInput: PartialEq + Debug + Clone {}
impl<T> TInput for T where T: PartialEq + Debug + Clone {}

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct Content<'input, T: TInput> {
    pub(crate) content: T,
    pub(crate) input: &'input str,
    pub(crate) left: usize,
    pub(crate) right: usize,
    pub(crate) data_quality: DataQuality,
}

impl<'input, T: TInput> Content<'input, T> {
    pub(crate) fn new(content: T, input: &'input str, dq: DataQuality) -> Self {
        Self {
            content,
            input,
            left: 0,
            right: input.len(),
            data_quality: dq,
        }
    }

    pub(crate) fn map<U: TInput, M: Fn(&T) -> U>(&self, mapper: M) -> Content<'input, U> {
        let Content {
            input,
            left,
            right,
            data_quality: dq,
            ..
        } = self.clone();
        Content {
            content: mapper(&self.content),
            input,
            left,
            right,
            data_quality: dq,
        }
    }

    pub(crate) fn input_string(&self) -> String {
        self.input[self.left..self.right].to_string()
    }
}

impl<'a, T, O> FromIterator<Content<'a, T>> for Content<'a, O>
where
    T: TInput,
    O: FromIterator<T> + TInput,
{
    fn from_iter<I: IntoIterator<Item = Content<'a, T>>>(iter: I) -> Self {
        let mut result = Content::<Vec<T>>::default();
        result.data_quality = NoValue;

        for Content {
            content,
            input,
            right,
            data_quality,
            ..
        } in iter
        {
            result.content.push(content);
            result.input = input;
            result.right = result.right.max(right);
            result.data_quality = result.data_quality.max(data_quality)
        }
        result.map(|items| items.clone().into_iter().collect())
    }
}

impl<'input, T: TInput + Default> Default for Content<'input, T> {
    fn default() -> Self {
        Self {
            content: T::default(),
            input: "",
            left: 0,
            right: 0,
            data_quality: Default::default(),
        }
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
