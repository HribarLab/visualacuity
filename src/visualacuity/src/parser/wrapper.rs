// Lalrpop generates a bunch of `...Parser` types as a pre-build step. Here we apply very thin
// wrappers by the same names for the sake of convenience, shimming in common interfaces for
// `.new()` and `.parse()`

use crate::parser::Content;
use crate::{ParsedItemCollection, VisualAcuityResult};

pub(crate) trait Parse<'a, T> {
    fn new() -> Self;
    fn parse(&'a self, s: &'a str) -> VisualAcuityResult<T>;
}

macro_rules! impl_parser {
    ($p:ty, $i:ty, $t:ty) => {
        impl<'a> Parse<'a, $t> for $p {
            fn new() -> Self {
                Self(<$i>::new())
            }

            fn parse(&'a self, orig: &'a str) -> VisualAcuityResult<$t> {
                let s = orig.to_lowercase();
                Ok(self.0.parse(orig, s.as_str())?)
            }
        }
    };
}

// Wrap these types so we can do trait implementation
pub(crate) struct ChartNotesParser(crate::parser::grammar::ChartNotesParser);
impl<'a> Parse<'a, Content<'a, ParsedItemCollection>> for ChartNotesParser {
    fn new() -> Self {
        Self(crate::parser::grammar::ChartNotesParser::new())
    }

    fn parse(&'a self, orig: &'a str) -> VisualAcuityResult<Content<'a, ParsedItemCollection>> {
        let s = orig.to_lowercase();
        let parsed = self.0.parse(orig, s.as_str())?;
        // Do a little switcheroo for lifetime reasons:
        Ok(Content {
            input: orig,
            ..parsed
        })
    }
}

pub(crate) struct KeyParser(crate::parser::key::KeyParser);
impl_parser!(
    KeyParser,
    crate::parser::key::KeyParser,
    crate::EntryMetadata
);

#[allow(dead_code)]
pub(crate) struct PlusLettersParser(crate::parser::grammar::PlusLettersParser);
impl_parser!(
    PlusLettersParser,
    crate::parser::grammar::PlusLettersParser,
    crate::ParsedItem
);

#[allow(dead_code)]
pub(crate) struct JaegerExactParser(crate::parser::grammar::JaegerExactParser);
impl_parser!(
    JaegerExactParser,
    crate::parser::grammar::JaegerExactParser,
    crate::ParsedItem
);

#[allow(dead_code)]
pub(crate) struct SnellenExactParser(crate::parser::grammar::SnellenExactParser);
impl_parser!(
    SnellenExactParser,
    crate::parser::grammar::SnellenExactParser,
    crate::ParsedItem
);

#[allow(dead_code)]
pub(crate) struct DistanceUnitsParser(crate::parser::grammar::DistanceUnitsParser);
impl_parser!(
    DistanceUnitsParser,
    crate::parser::grammar::DistanceUnitsParser,
    crate::DistanceUnits
);
