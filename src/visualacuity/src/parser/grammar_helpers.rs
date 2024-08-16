use std::fmt::Debug;
use std::str::FromStr;

use itertools::traits::HomogeneousTuple;
use itertools::Itertools;
use lalrpop_util::lexer::Token;
use lalrpop_util::ErrorRecovery;
use lalrpop_util::ParseError::{UnrecognizedEof, UnrecognizedToken};

use crate::charts::ChartRow;
use crate::parser::decorator::Content;
use crate::ParsedItem::{Text, Unhandled};
use crate::VisualAcuityError::ParseError;
use crate::{ParsedItem, ParsedItemCollection, VisualAcuityResult};

pub(crate) fn merge_consecutive_texts<'a>(
    items: Vec<Content<'a, ParsedItem>>,
) -> Content<'a, ParsedItemCollection> {
    items
        .into_iter()
        .map(validate)
        .fold(vec![], |mut accum, next| {
            if next.content == Text(String::default()) {
                return accum;
            }
            let Some(prev) = accum.pop() else {
                accum.push(next);
                return accum;
            };
            let to_append = match (&prev.content, &next.content) {
                (Text(..), Text(..)) => vec![merge_text(prev, next)],
                _ => vec![prev, next],
            };
            to_append
                .into_iter()
                .filter(|input| input != &Text("".to_string()))
                .for_each(|input| accum.push(input));
            accum
        })
        .into_iter()
        .collect()
}

fn validate<'a>(input: Content<'a, ParsedItem>) -> Content<'a, ParsedItem> {
    /// Turn a ParsedItem back into ParsedItem::Text() if it's not a valid chart row
    use ParsedItem::*;
    match &input.content {
        SnellenFraction(s) | Jaeger(s) | Teller(s) | ETDRS(s) | NearTotalLoss(s, _) => {
            match ChartRow::find(s) {
                None => input.map(|_| Text(input.input_string())),
                Some(_) => input,
            }
        }
        VisualResponse(_) => input,
        CrossReferenceItem(_) => input,
        PlusLettersItem(_) => input,
        NotTakenItem(_) => input,
        DistanceItem(_) => input,
        LateralityItem(_) => input,
        CorrectionItem(_) => input,
        PinHoleItem(_) => input,
        Text(_) => input,
        Unhandled(_) => input,
    }
}

fn merge_text<'a>(
    prev: Content<'a, ParsedItem>,
    next: Content<'a, ParsedItem>,
) -> Content<'a, ParsedItem> {
    let merged = Content {
        left: prev.left,
        ..next
    };
    Content {
        content: Text(merged.to_string()),
        ..merged
    }
}

pub(crate) fn extract_floats<T: HomogeneousTuple>(s: &str) -> VisualAcuityResult<T>
where
    T::Item: FromStr,
    <T::Item as FromStr>::Err: Debug,
{
    match iter_decimals(s).next_tuple() {
        Some(t) => Ok(t),
        None => Err(ParseError(format!("{s}"))),
    }
}

pub(crate) fn extract_float<T>(s: &str) -> VisualAcuityResult<T>
where
    T: FromStr + ToString,
    <T as FromStr>::Err: Debug,
{
    Ok(iter_decimals(s).exactly_one()?)
}

fn iter_decimals<T: FromStr>(s: &str) -> impl Iterator<Item = T> + '_ {
    s.split(|c: char| !(c.is_numeric() || c == '.'))
        .filter(|&s| s != "")
        .filter_map(|s| s.parse::<T>().ok())
        .into_iter()
}

pub(crate) fn handle_error<'a>(
    boxed_value: Content<'a, ErrorRecovery<usize, Token, &'a str>>,
) -> Content<'a, ParsedItem> {
    match boxed_value.content.error {
        UnrecognizedEof { .. } => boxed_value.map(|_| Text(boxed_value.input_string())),
        UnrecognizedToken { .. } => boxed_value.map(|_| Text(boxed_value.input_string())),
        _ => boxed_value.map(|error_recovery| Unhandled(format!("{:?}", error_recovery))),
    }
}

#[cfg(test)]
mod tests {
    use crate::dataquality::DataQuality;
    use crate::parser::decorator::Content;
    use crate::parser::grammar_helpers::merge_consecutive_texts;
    use crate::ParsedItem::*;

    #[test]
    fn test_merge_texts() {
        let test_cases = [(
            vec![
                Content {
                    content: Text("asdf".to_string()),
                    left: 0,
                    right: 4,
                    input: "asdf qwerty",
                    data_quality: DataQuality::ConvertibleFuzzy,
                },
                Content {
                    content: Text("qwerty".to_string()),
                    left: 5,
                    right: 11,
                    input: "asdf qwerty",
                    data_quality: DataQuality::ConvertibleFuzzy,
                },
            ],
            vec![Text("asdf qwerty".to_string())],
        )];
        for (va, expected) in test_cases {
            let actual = merge_consecutive_texts(va).content;
            assert_eq!(actual, expected.into_iter().collect());
        }
    }
}
