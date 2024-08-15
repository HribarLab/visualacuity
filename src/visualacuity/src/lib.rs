extern crate lalrpop_util;
extern crate visualacuity_proc_macro;

use itertools::Itertools;
use lazy_static::lazy_static;

pub use dataquality::DataQuality;
pub use distanceunits::*;
pub use structure::*;
pub use visit::{EntryMetadata, Visit, VisitNote};
use visitinput::ColumnMerger;
pub use visitinput::VisitInput;

use crate::cache::LruCacher;
pub use crate::errors::{OptionResult, VisualAcuityError, VisualAcuityResult};
use crate::parser::*;
pub use crate::visit::metadata::*;
use crate::ParsedItem::*;

mod cache;
mod charts;
mod distanceunits;
pub(crate) mod errors;
mod logmar;
pub(crate) mod parser;
mod snellen_equivalent;
pub(crate) mod structure;
mod types;
mod visit;
mod visitinput;

mod dataquality;
mod helpers;
mod macros;
#[cfg(test)]
pub(crate) mod tests;

pub struct Parser {
    notes_parser: &'static ChartNotesParser,
    key_parser: &'static KeyParser,
    cache: LruCacher<(String, String, String), VisualAcuityResult<Option<VisitNote>>>,
    // parse_cache: LruCacher<String, (DataQuality, ParsedItemCollection)>,
    // key_cache: LruCacher<String, VisualAcuityResult<EntryMetadata>>,
    column_merger: ColumnMerger,
}

impl Parser {
    pub fn new() -> Self {
        lazy_static! {
            static ref CHART_NOTES_PARSER: ChartNotesParser = ChartNotesParser::new();
            static ref KEY_PARSER: KeyParser = KeyParser::new();
        }
        let cache_size = 999;
        // let parse_cache = LruCacher::new(cache_size);
        let cache = LruCacher::new(cache_size);
        // let key_cache = LruCacher::new(cache_size);
        let column_merger = ColumnMerger::new(cache_size);
        Self {
            notes_parser: &CHART_NOTES_PARSER,
            key_parser: &KEY_PARSER,
            cache,
            column_merger,
        }
    }

    pub fn parse_visit(&self, visit_notes: VisitInput) -> VisualAcuityResult<Visit> {
        use VisualAcuityError::*;

        let merged = self.column_merger.merge_plus_columns(visit_notes.into());
        let (parsed_visit_notes, errors): (_, Vec<_>) = merged
            .into_iter()
            .map(|(key, (text, text_plus))| {
                let visit_note = self.parse_visit_note(key.as_str(), (&*text, &*text_plus))?;
                Ok((key, visit_note))
            })
            .partition_result();

        if errors.len() > 0 {
            return Err(MultipleErrors(errors));
        }

        Ok(Visit(parsed_visit_notes))
    }

    fn parse_visit_note(
        &self,
        key: &str,
        (text, text_plus): (&str, &str),
    ) -> VisualAcuityResult<Option<VisitNote>> {
        let key = key.trim();
        let text = text.trim();
        let text_plus = text_plus.trim();

        if (text, text_plus) == ("", "") {
            return Ok(None);
        }

        let cache_key = (
            key.to_lowercase(),
            text.to_lowercase(),
            text_plus.to_lowercase(),
        );

        self.cache.get(&cache_key, || {
            let parsed_text = self.parse_text(text);
            let parsed_text_plus = self.parse_text(text_plus);
            let visit_metadata = self.parse_key(key)?;
            VisitNote::new(visit_metadata, parsed_text, parsed_text_plus).map(Some)
        })
    }

    fn parse_text<'input>(&self, notes: &'input str) -> Content<'input, ParsedItemCollection> {
        let notes = notes.trim();
        let (dq, content) = match self.notes_parser.parse(notes) {
            Ok(Content {
                content,
                data_quality: dq,
                ..
            }) => (dq, content),
            Err(e) => (
                DataQuality::ConvertibleFuzzy,
                ParsedItemCollection(vec![Unhandled(format!(" {e}"))]),
            ),
        };
        Content::new(content, notes, dq)
    }

    fn parse_key<'input>(&self, key: &'input str) -> VisualAcuityResult<EntryMetadata> {
        Ok(self.key_parser.parse(key)?)
    }
}
