use std::collections::HashMap;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use derive_more::IntoIterator;
use crate::{Correction, CorrectionItem, DistanceOfMeasurement, Laterality, ParsedItem, ParsedItemCollection, VisualAcuityResult};
use crate::logmar::{LogMarBase, LogMarPlusLetters};
use crate::ParsedItem::*;
use crate::snellen_equivalent::SnellenEquivalent;
use crate::structure::{Method, PinHole, Disambiguate};


#[derive(IntoIterator, PartialEq, Debug, Clone)]
pub struct Visit<'input>(pub(crate) HashMap<&'input str, VisitNote<'input>>);

#[derive(PartialEq, Debug, Clone)]
pub struct VisitNote<'input> {
    pub text: &'input str,
    pub text_plus: &'input str,

    pub laterality: Laterality,
    pub distance_of_measurement: DistanceOfMeasurement,
    pub correction: Correction,
    pub pinhole: PinHole,

    pub method: Method,
    pub plus_letters: Vec<i32>,

    pub extracted_value: String,
    pub snellen_equivalent: VisualAcuityResult<Option<(u16, u16)>>,
    pub log_mar_base: VisualAcuityResult<Option<f64>>,
    pub log_mar_base_plus_letters: VisualAcuityResult<Option<f64>>,
}

impl<'a> VisitNote<'a> {
    pub(crate) fn new(
        text: &'a str,
        text_plus: &'a str,
        parsed_key: ParsedItemCollection<'a>,
        parsed_notes: ParsedItemCollection<'a>,
    ) -> VisualAcuityResult<Self> {
        let combined = ParsedItemCollection(parsed_key.into_iter().chain(parsed_notes.clone().into_iter()).collect());
        let mut pinholes = vec![];
        let mut acuity_items = vec![];
        let mut other_observations = vec![];
        let mut lateralities = vec![];
        let mut distances = vec![];
        let mut corrections = vec![];
        let mut plus_letters = vec![];
        let mut methods = vec![];
        for item in combined.into_iter() {
            match item {
                Snellen { .. } => { acuity_items.push(item); methods.push(Method::Snellen); }
                Jaeger { .. } => { acuity_items.push(item); methods.push(Method::Jaeger); }
                Teller { .. } => { acuity_items.push(item); methods.push(Method::Teller); }
                ETDRS { .. } => { acuity_items.push(item); methods.push(Method::ETDRS); }
                LowVision { .. } => { acuity_items.push(item); methods.push(Method::LowVision);  }
                PinHoleEffectItem { .. } => { other_observations.push(item); }
                BinocularFixation(_) => { other_observations.push(item); }
                NotTakenItem(_) => { other_observations.push(item); }
                PlusLettersItem(value) => plus_letters.push(value),
                DistanceItem(value) => distances.push(value),
                LateralityItem(value) => lateralities.push(value),
                CorrectionItem(value) => corrections.push(value),
                PinHoleItem(value) => pinholes.push(value),
                Text(_) => {}
                Unhandled(_) => {}
            }
        }

        let acuity_item = acuity_items.into_iter()
            .rev()  // Take the *last* equivalent thing (e.g. ETDRS)
            .unique_by(|acuity| acuity.snellen_equivalent())
            .collect_vec() // Put it back in order
            .into_iter()
            .rev()
            .at_most_one();

        let base_item = match acuity_item {
            // If no acuity is present, see if we have another kind of observation
            Ok(None) => Ok(other_observations.into_iter().next()),
            acuity_item => acuity_item,
        };

        let (
            extracted_value,
            snellen_equivalent,
            log_mar_base,
            log_mar_base_plus_letters,
            method
        ) = match base_item {
            Ok(Some(value)) => (
                extract_value(&value),
                Ok(value.snellen_equivalent().ok()), // Error here is just NoSnellenEquivalent.
                Ok(value.log_mar_base().ok()), // Error here is just NoSnellenEquivalent.
                value.log_mar_plus_letters(&plus_letters).map(|s| Some(s)),
                get_method(&value)
            ),
            Ok(None) => (
                format!(""),
                Ok(None),
                Ok(None),
                Ok(None),
                Method::disambiguate(methods)
            ),
            Err(e) => (
                format!("Error"),
                Err(e.clone().into()),
                Err(e.clone().into()),
                Err(e.into()),
                Method::disambiguate(methods)
            )
        };

        let distance_of_measurement = DistanceOfMeasurement::disambiguate(distances);
        let laterality = Laterality::disambiguate(lateralities);
        let correction = Correction::disambiguate(corrections);
        let pinhole = PinHole::disambiguate(pinholes);

        Ok(VisitNote {
            text, text_plus, distance_of_measurement, correction, pinhole, method, laterality,
            plus_letters, extracted_value, snellen_equivalent, log_mar_base, log_mar_base_plus_letters
        })
    }
}

fn extract_value(item: &ParsedItem) -> String {
    match item {
        Snellen(row) => format!("20/{}", *row as u16),
        Jaeger(row) => format!("{row:?}").replace("PLUS", "+"),
        ETDRS{letters} => format!("{letters} letters"),
        PinHoleItem(effect) => format!("{effect:?}"),
        Teller { card, .. } => format!("card {card:?}"),
        LowVision { method, .. } => format!("{method:?}"),
        PinHoleEffectItem(effect) => format!("{effect:?}"),
        BinocularFixation(preference) => format!("{preference:?}"),
        NotTakenItem(reason) => format!("{reason:?}"),
        _ => format!(""),
    }
}

fn get_method(item: &ParsedItem) -> Method {
    match item {
        Snellen { .. } => Method::Snellen,
        Jaeger { .. } => Method::Jaeger,
        Teller { .. } => Method::Teller,
        ETDRS { .. } => Method::ETDRS,
        LowVision { .. } => Method::LowVision,
        PinHoleEffectItem(_) => Method::PinHole,
        BinocularFixation(_) => Method::Binocular,
        NotTakenItem(_) => Method::NotTaken,
        _ => Method::Unknown,
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct StandardizedVisualAcuity {
    pub snellen_equivalent: (u16, u16),
    pub log_mar: f64,
    pub log_mar_base: f64,
    pub log_mar_plus_letters: f64,
}


lazy_static!{
    static ref RE_PLUS_COLUMN: Regex = Regex::new(r"(?i)^(.*?)\s*(\+/-|\+|\splus)$").expect("");
}

pub(crate) fn merge_plus_columns<T>(notes: HashMap<&str, Vec<T>>) -> HashMap<&str, Vec<T>> {
    // let keys: HashSet<_> = notes.keys().into_iter().cloned().collect();

    notes.into_iter()
        .sorted_by_key(|(key, _)| *key)
        .map(|(key, value)| {
            let parent_key = RE_PLUS_COLUMN.captures(key)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str());

            (parent_key.unwrap_or(key), value)
        })
        .group_by(|(key, _)| *key)
        .into_iter()
        .map(|(key, group)| {
            (key, group.flat_map(|(_, v)| v).collect())
        })
        .collect()
}
