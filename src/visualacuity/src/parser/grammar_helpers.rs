use std::fmt::Debug;
use std::str::FromStr;
use crate::{Input, ParsedItem, ParsedItemCollection, VisualAcuityResult};
use crate::ParsedItem::{Text, Unhandled};

use itertools::Itertools;
use itertools::traits::HomogeneousTuple;
use lalrpop_util::ErrorRecovery;
use lalrpop_util::lexer::Token;
use lalrpop_util::ParseError::{UnrecognizedEof, UnrecognizedToken};
use crate::charts::ChartRow;
use crate::VisualAcuityError::ParseError;

pub(crate) fn merge_consecutive_texts<'a>(items: Vec<Input<'a, ParsedItem>>) -> ParsedItemCollection {
    items.into_iter()
        .map(validate)
        .fold(vec![], |mut accum, next| {
            let Some(prev) = accum.pop() else { accum.push(next); return accum; };
            let to_append = match (&prev.content, &next.content) {
                (Text(..), Text(..)) => vec![merge_text(prev, next)],
                _ => vec![prev, next]
            };
            to_append.into_iter()
                .filter(|input| input != &Text("".to_string()))
                .for_each(|input| accum.push(input));
            accum
        })
        .into_iter()
        .map(|item| item.content)
        .collect()
}

fn validate<'a>(input: Input<'a, ParsedItem>) -> Input<'a, ParsedItem> {
    /// Turn a ParsedItem back into ParsedItem::Text() if it's not a valid chart row
    use ParsedItem::*;
    match &input.content {
        SnellenFraction(s)
        | Jaeger(s)
        => match ChartRow::find(&s) {
            None => Input { content: Text(input.to_string()), ..input },
            Some(_) => input
        }
        _ => input
    }
}

fn merge_text<'a>(prev: Input<'a, ParsedItem>, next: Input<'a, ParsedItem>) -> Input<'a, ParsedItem> {
    let merged = Input { left: prev.left, ..next};
    Input { content: Text(merged.to_string()), ..merged}
}

pub(crate) fn extract_integers<T: HomogeneousTuple>(s: &str) -> Option<T>
    where T::Item: FromStr, <T::Item as FromStr>::Err: Debug
{
    iter_integer(s).next_tuple()
}

fn iter_integer<T: FromStr>(s: &str) -> impl Iterator<Item=T> + '_ {
    s.split(|c: char| !c.is_numeric())
        .filter(|&s| s != "")
        .filter_map(|s| s.parse::<T>().ok())
        .into_iter()
}

pub(crate) fn extract_floats<T: HomogeneousTuple>(s: &str) -> VisualAcuityResult<T>
    where T::Item: FromStr, <T::Item as FromStr>::Err: Debug
{
    match iter_decimals(s).next_tuple() {
        Some(t) => Ok(t),
        None => Err(ParseError(format!("{s}")))
    }
}

pub(crate) fn extract_float<T>(s: &str) -> VisualAcuityResult<T>
    where T: FromStr + Debug, <T as FromStr>::Err: Debug
{
    Ok(iter_decimals(s).exactly_one()?)
}

fn iter_decimals<T: FromStr>(s: &str) -> impl Iterator<Item=T> + '_ {
    s.split(|c: char| !(c.is_numeric() || c == '.'))
        .filter(|&s| s != "")
        .filter_map(|s| s.parse::<T>().ok())
        .into_iter()
}

pub(crate) fn handle_error<'a>(boxed_value: Input<'a, ErrorRecovery<usize, Token, &'a str>>) -> ParsedItem {
    let text = boxed_value.input[boxed_value.left..boxed_value.right].to_string();
    match boxed_value.content.error {
        UnrecognizedEof { .. } => Text(text),
        UnrecognizedToken { .. } => Text(text),
        _ => Unhandled(format!("{:?}", boxed_value.content)),
    }
}

#[cfg(test)]
mod tests {
    use crate::Input;
    use crate::parser::grammar_helpers::merge_consecutive_texts;
    use crate::ParsedItem::*;

    #[test]
    fn test_merge_texts() {
        let test_cases = [
            (
                vec![
                    Input { content: Text("asdf".to_string()), left: 0, right: 4, input: "asdf qwerty" },
                    Input { content: Text("qwerty".to_string()), left: 5, right: 11, input: "asdf qwerty" },
                ],
                vec![
                    Text("asdf qwerty".to_string()),
                ],
            )
        ];
        for (va, expected) in test_cases {
            let actual = merge_consecutive_texts(va);
            assert_eq!(actual, expected.into_iter().collect());
        }
    }
}