use std::fmt::Debug;
use std::str::FromStr;
use crate::{Input, ParsedItem, ParsedItemCollection, VisualAcuityResult};
use crate::ParsedItem::{Text, Unhandled};

use itertools::Itertools;
use itertools::traits::HomogeneousTuple;
use lalrpop_util::ErrorRecovery;
use lalrpop_util::lexer::Token;
use lalrpop_util::ParseError::{UnrecognizedEof, UnrecognizedToken};
use crate::VisualAcuityError::ParseError;

pub(crate) fn merge_consecutive_texts<'a>(items: Vec<Input<'a, ParsedItem>>) -> ParsedItemCollection {
    items.into_iter()
        .map(|item| vec![item])
        .reduce(|mut accum, mut next| {
            let Input{content: a, left, .. } = accum.last().expect("86784!");
            let next = next.pop().expect("aifewjo!");
            let Input{content: b, right, .. } = &next;
            let (left, right) = (*left, *right);
            match (&a, &b) {
                (&Text(..), &Text(..)) => {
                    let Input{ input, .. } = accum.pop().unwrap();
                    let content = Text(input[left..right].to_string());
                    accum.push(Input { content, left, right, input })
                }
                _ => accum.push(next)
            };
            accum
        })
        .unwrap_or(vec![])
        .into_iter()
        .filter(|va| va != &Text("".to_string()))
        .map(|item| item.content)
        .collect()
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