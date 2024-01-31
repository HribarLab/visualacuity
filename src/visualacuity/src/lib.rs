#[macro_use]
extern crate num_derive;
extern crate lalrpop_util;

pub(crate) mod errors;
pub(crate) mod parser;
pub(crate) mod structure;
mod logmar;
mod snellen_equivalent;
mod visit;
mod cache;
mod visitinput;

use itertools::Itertools;
pub use visit::{Visit, VisitNote};
use visitinput::ColumnMerger;
use crate::ParsedItem::*;
use crate::VisualAcuityError::{*};
use crate::cache::LruCacher;


pub use crate::errors::{VisualAcuityError, VisualAcuityResult};

pub use structure::{
    Correction,
    DistanceOfMeasurement,
    FixationPreference,
    Input,
    JaegerRow,
    Laterality,
    LowVisionMethod,
    Method,
    NotTakenReason,
    ParsedItem,
    ParsedItemCollection,
    PinHole,
    PinHoleEffect,
    SnellenRow,
};
pub use visitinput::{VisitInput};

pub struct Parser {
    notes_parser: parser::grammar::ChartNotesParser,
    parse_cache: LruCacher<String, ParsedItemCollection>,
    column_merger: ColumnMerger
}

impl Parser {
    pub fn new() -> Self {
        let cache_size = 9999;
        let parse_cache = LruCacher::new(cache_size);
        let column_merger = ColumnMerger::new(cache_size);
        let notes_parser = parser::grammar::ChartNotesParser::new();
        Self { notes_parser, parse_cache, column_merger }
    }

    pub fn parse_visit(
        &self,
        visit_notes: VisitInput,
    ) -> VisualAcuityResult<Visit>
    {
        let merged = self.column_merger.merge_plus_columns(visit_notes.into());
        let (parsed_visit_notes, errors): (_, Vec<_>) = merged
            .into_iter()
            .filter(|(_, (text, text_plus))| !(text.trim().is_empty() && text_plus.is_empty()))
            .map(|(key, (text, text_plus))| {
                let visit_note = self.parse_visit_note(key.as_str(), (&*text, &*text_plus))?;
                Ok((key, visit_note))
            })
            .partition_result();

        if errors.len() > 0 {
            return Err(MultipleErrors(errors))
        }

        Ok(Visit(parsed_visit_notes))
    }

    fn parse_visit_note(
        &self,
        key: &str,
        (text, text_plus): (&str, &str)
    ) -> VisualAcuityResult<VisitNote> {
        let parsed_text = self.parse_text(text);
        let parsed_text_plus = self.parse_text(text_plus);
        let parsed_notes = [parsed_text, parsed_text_plus].into_iter().flatten().collect();
        let parsed_key = self.parse_text(key);
        VisitNote::new(text.to_string(), text_plus.to_string(), parsed_key, parsed_notes)
    }

    fn parse_text(&self, notes: &str) -> ParsedItemCollection {
        self.parse_cache.get(&notes.trim().to_string(), || {
            match self.notes_parser.parse(notes.trim()) {
                Ok(p) => p,
                Err(e) => [Unhandled(format!(" {e}"))].into_iter().collect()
            }
        })
    }
}
