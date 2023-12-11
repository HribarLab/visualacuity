#[macro_use]
extern crate num_derive;
extern crate lalrpop_util;

pub(crate) mod errors;
pub(crate) mod parser;
pub(crate) mod structure;
mod logmar;
mod snellen_equivalent;
mod visit;

use std::collections::HashMap;
use itertools::Itertools;
pub use visit::{Visit, VisitNote};
pub use logmar::LogMarEstimate;


pub use crate::errors::{VisualAcuityError, VisualAcuityResult};
use crate::visit::merge_plus_columns;
use crate::ParsedItem::*;
use crate::VisualAcuityError::{*};

pub use structure::{
    ParsedItem,
    ParsedItemCollection,
    FixationPreference,
    Input,
    JaegerRow,
    LowVisionMethod,
    PinHoleEffect,
    SnellenRow,
    Correction,
    DistanceOfMeasurement,
    Laterality,
    Method,
    PinHole,
    NotTakenReason,
};
pub struct Parser {
    notes_parser: parser::grammar::ChartNotesParser,
}

impl Parser {
    pub fn new() -> Self {
        let notes_parser = parser::grammar::ChartNotesParser::new();
        Self { notes_parser }
    }

    pub fn parse_visit<'input>(
        &self,
        visit_notes: HashMap<&'input str, &'input str>,
    ) -> VisualAcuityResult<Visit<'input>> {
        let visit_notes = visit_notes.into_iter()
            .map(|(key, input)| (key, vec![input]))
            .collect();
        let (parsed_visit_notes, errors) = merge_plus_columns(visit_notes)
            .into_iter()
            .map(|(key, notes)| -> VisualAcuityResult<_> {
                let parsed_notes = ParsedItemCollection(
                    notes.clone().into_iter()
                        .flat_map(|n| self.parse_notes(n))
                        .collect()
                );
                let parsed_key = if parsed_notes.len() > 0 {
                    self.parse_notes(key)
                } else {
                    // Don't parse information from the key if the text is blank
                    ParsedItemCollection(vec![])
                };

                let mut text_iter = notes.into_iter();
                let text = text_iter.next().unwrap_or("");
                let text_plus = text_iter.next().unwrap_or("");

                let visit_note = VisitNote::new(text, text_plus, parsed_key, parsed_notes)?;
                Ok((key, visit_note))
            })
            .partition_result::<Vec<_>, Vec<_>, _, _>();

        if errors.len() > 0 {
            return Err(MultipleErrors(errors))
        }

        Ok(Visit(parsed_visit_notes.into_iter().collect()))
    }

    fn parse_notes<'input>(&self, notes: &'input str) -> ParsedItemCollection<'input> {
        if notes.chars().all(|c| c.is_whitespace()) {
            ParsedItemCollection(vec![])
        }
        else {
            match self.notes_parser.parse(notes) {
                Ok(p) => p,
                Err(e) => ParsedItemCollection(vec![Unhandled(format!("<ERROR> {e}"))])
            }
        }
    }
}
