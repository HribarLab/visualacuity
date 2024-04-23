// Lalrpop generates a bunch of `...Parser` types as a pre-build step. Here we apply very thin
// wrappers by the same names for the sake of convenience, shimming in common interfaces for
// `.new()` and `.parse()`

type E<'a> = lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'a>, &'static str>;

pub(crate) trait Parse<T> {
    fn parse<'a>(&'a self, orig: &'a str, s: &'a str) -> Result<T, E>;
    fn new() -> Self;
}

macro_rules! impl_parser {
    ($p:ty, $i:ty, $t:ty) => {
        impl Parse<$t> for $p {
            fn new() -> Self {
                Self(<$i>::new())
            }

            fn parse<'a>(&'a self, orig: &'a str, s: &'a str) -> Result<$t, E> {
                self.0.parse(orig, s)
            }
        }
    };
}

// Wrap these types so we can do trait implementation
pub(crate) struct ChartNotesParser(crate::parser::grammar::ChartNotesParser);
impl_parser!(ChartNotesParser, crate::parser::grammar::ChartNotesParser, crate::ParsedItemCollection);

pub(crate) struct PlusLettersParser(crate::parser::grammar::PlusLettersParser);
impl_parser!(PlusLettersParser, crate::parser::grammar::PlusLettersParser, crate::ParsedItem);

pub(crate) struct JaegerExactParser(crate::parser::grammar::JaegerExactParser);
impl_parser!(JaegerExactParser, crate::parser::grammar::JaegerExactParser, crate::ParsedItem);

pub(crate) struct SnellenExactParser(crate::parser::grammar::SnellenExactParser);
impl_parser!(SnellenExactParser, crate::parser::grammar::SnellenExactParser, crate::ParsedItem);

pub(crate) struct DistanceUnitsParser(crate::parser::grammar::DistanceUnitsParser);
impl_parser!(DistanceUnitsParser, crate::parser::grammar::DistanceUnitsParser, crate::DistanceUnits);
