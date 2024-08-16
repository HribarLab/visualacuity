use std::collections::BTreeMap;
use std::iter::repeat;
use std::str::FromStr;

use crate::errors::OptionResult;
use crate::VisualAcuityError::UnknownError;
use crate::{VisualAcuityError, VisualAcuityResult};

#[derive(Default, Clone, PartialEq, Debug)]
pub(crate) struct TestCaseRow {
    pub(crate) line_number: usize,
    fields: BTreeMap<String, String>,
}

impl TestCaseRow {
    // Get a string from a row. Blanks return empty strings. Invalid columns return Err().
    pub(crate) fn get(&self, key: &str) -> VisualAcuityResult<String> {
        match self.fields.get(key) {
            Some(v) => Ok(v.to_string()),
            None => Err(UnknownError(format!("Invalid column name: {key}"))),
        }
    }

    /// Convenience function to do a FromStr::parse<T>() when reading a field
    pub(crate) fn parse<T>(&self, key: &str) -> OptionResult<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Into<VisualAcuityError>,
    {
        match self.get(key).into() {
            OptionResult::None => OptionResult::None,
            OptionResult::Err(e) => OptionResult::Err(e),
            OptionResult::Some(s) => match s.trim() {
                "" => OptionResult::None,
                _ => s.parse::<T>().into(),
            },
        }
    }

    pub(crate) fn read_file(content: &str) -> impl Iterator<Item = TestCaseRow> + '_ {
        static COMMENT_CHAR: &str = "#";

        let mut lines = content
            .lines()
            .map(|s| s.split('\t').map(|s| s.to_string()));
        let header = lines.next().expect("Empty TSV!").into_iter();
        lines
            .enumerate()
            .filter(|(_, v)| {
                !v.clone()
                    .next()
                    .unwrap_or_default()
                    .starts_with(COMMENT_CHAR)
            })
            .map(move |(i, values)| TestCaseRow {
                line_number: i + 1,
                fields: header
                    .clone()
                    .zip(values.chain(repeat(String::default())))
                    .collect(),
            })
    }
}
