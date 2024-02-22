use std::collections::BTreeMap;
use itertools::Itertools;
use derive_more::IntoIterator;
use crate::{Correction, CorrectionItem, DistanceOfMeasurement, Laterality, ParsedItem, ParsedItemCollection, VisualAcuityResult};
use crate::logmar::{LogMarBase, LogMarPlusLetters};
use crate::ParsedItem::*;
use crate::snellen_equivalent::SnellenEquivalent;
use crate::structure::{Disambiguate, Fraction, Method, PinHole};

/// The public return type representing a parsed & processed set of EHR notes for a patient visit
#[derive(IntoIterator, PartialEq, Debug, Clone)]
pub struct Visit(pub(crate) BTreeMap<String, VisitNote>);

/// The parsed & processed observations from an EHR field (with "plus" columns merged if possible)
#[derive(PartialEq, Debug, Clone)]
pub struct VisitNote {
    /// The contents of the original text field
    pub text: String,
    /// The contents of the associated "Plus" field, when available
    pub text_plus: String,

    /// The laterality (typically retrieved from the field name)
    pub laterality: Laterality,
    /// The distance of measurement (typically retrieved from the field name)
    pub distance_of_measurement: DistanceOfMeasurement,
    /// Whether correction was used (typically retrieved from the field name)
    pub correction: Correction,
    /// Whether the observation involved pin-hole methods (typically retrieved from field name)
    pub pinhole: PinHole,

    /// The method used during observation
    pub method: Method,
    ///  When a patient reads a partial line, how many letters were indicated in the note  (+/-)
    pub plus_letters: Vec<i32>,

    /// The "normalized" text describing the visual acuity observation
    pub extracted_value: String,
    /// The Snellen equivalent of the visual acuity (if available), expressed as a fraction
    pub snellen_equivalent: VisualAcuityResult<Option<Fraction>>,
    /// The LogMAR equivalent of the visual acuity (if available), not considering partial lines
    pub log_mar_base: VisualAcuityResult<Option<f64>>,
    /// The LogMAR equivalent of the visual acuity (if available), with consideration of partial lines
    pub log_mar_base_plus_letters: VisualAcuityResult<Option<f64>>,
}

impl VisitNote {
    /// Given a variable length set of `ParsedItem`s, determine which items represent the visual
    /// acuity measurements etc. The task here mostly has to do with prioritization/disambiguation.
    pub(crate) fn new(
        text: String,
        text_plus: String,
        parsed_key: ParsedItemCollection,
        parsed_notes: ParsedItemCollection,
    ) -> VisualAcuityResult<Self> {
        let sifted = SiftedParsedItems::sift(parsed_key, parsed_notes);

        let base_acuity = base_acuity(&sifted.acuities, &sifted.other_observations);
        let log_mar_base = map_ok_some(&base_acuity, |v| v.log_mar_base());
        let log_mar_base_plus_letters = map_ok_some(&base_acuity, |v| v.log_mar_plus_letters(&sifted.plus_letters));
        let method = get_method(&base_acuity, &sifted.acuities);
        let extracted_value = extract_value(&base_acuity);
        let distance_of_measurement = DistanceOfMeasurement::disambiguate(&sifted.distances);
        let correction = Correction::disambiguate(&sifted.corrections);
        let pinhole = PinHole::disambiguate(&sifted.pinholes);
        let laterality = Laterality::disambiguate(&sifted.lateralities);
        let snellen_equivalent = map_ok_some(&base_acuity, |v| v.snellen_equivalent());
        let plus_letters = sifted.plus_letters;

        Ok(VisitNote {
            text, text_plus, extracted_value, distance_of_measurement, correction, pinhole,
            laterality, plus_letters, method, snellen_equivalent, log_mar_base, log_mar_base_plus_letters
        })
    }
}

/// Given `ParsedItem`s, determine which one reperesents a "base acuity." If none are present,
/// consider "other observations" (e.g. binocular fixation) that might be the primary observation.
fn base_acuity(acuities: &Vec<ParsedItem>, other_observations: &Vec<ParsedItem>) -> VisualAcuityResult<Option<ParsedItem>> {
    let unique_acuities = acuities.iter()
        .rev()  // Take the *last* equivalent thing (e.g. ETDRS)
        .unique_by(|&acuity| acuity.snellen_equivalent())
        .collect_vec();

    let acuity_item = unique_acuities.into_iter()
        .rev()
        .at_most_one()?;

    match acuity_item {
        // If no acuity is present, see if we have another kind of observation
        None => Ok(other_observations.first().cloned()),
        acuity_item => Ok(acuity_item.cloned()),
    }
}

// The `Ok(Some(value))` structure is pretty inconvenient, and should maybe be reconsidered, but
// here's a convenience function to ease the pain a bit.
fn map_ok_some<T, F>(base_acuity: &VisualAcuityResult<Option<ParsedItem>>, f: F) -> VisualAcuityResult<Option<T>>
    where F: Fn (&ParsedItem) -> VisualAcuityResult<T>
{
    match base_acuity.as_ref()? {
        Some(ref value) => Ok(Some(f(value)?)),
        None => Ok(None)
    }
}

/// Retrieve the "normalized" text representing the primary observation in a given EHR note.
fn extract_value(item: &VisualAcuityResult<Option<ParsedItem>>) -> String {
    let Ok(item) = item else { return format!("Error") };
    let Some(item) = item else { return format!("") };
    item.to_string()
}

/// Determine the method used for the base observation.
fn get_method(base_acuity: &VisualAcuityResult<Option<ParsedItem>>, other_acuities: &Vec<ParsedItem>) -> Method {
    match base_acuity {
        Ok(Some(ref value)) => value.clone().into(),
        _ => Method::disambiguate(&other_acuities.iter().cloned().map_into())
    }
}

/// After text has been parsed, we have a list of `ParsedItem` objects, each of which may contain
/// a variety of kinds of information. This interstitial structure organizes the items by category,
/// which facilitates identifying & disambiguating key information.
#[derive(Default)]
struct SiftedParsedItems {
    acuities: Vec<ParsedItem>,
    other_observations: Vec<ParsedItem>,
    plus_letters: Vec<i32>,
    lateralities: Vec<Laterality>,
    distances: Vec<DistanceOfMeasurement>,
    corrections: Vec<Correction>,
    pinholes: Vec<PinHole>,
    unhandled: Vec<ParsedItem>,
}

impl SiftedParsedItems {
    /// Iterates through parsed items, assigning each variant of `ParsedItem` into a bin/category.
    fn sift(parsed_key: ParsedItemCollection, parsed_notes: ParsedItemCollection) -> Self {
        let mut result = Self::default();
        for item in [parsed_key, parsed_notes].into_iter().flatten() {
            match item {
                SnellenFraction { .. } => result.acuities.push(item),
                Jaeger { .. } => result.acuities.push(item),
                TellerCard { .. } => result.acuities.push(item),
                TellerCyCm { .. } => result.acuities.push(item),
                ETDRS { .. } => result.acuities.push(item),
                LowVision { .. } => result.acuities.push(item),
                PinHoleEffectItem { .. } => result.other_observations.push(item),
                BinocularFixation(_) => result.other_observations.push(item),
                NotTakenItem(_) => result.other_observations.push(item),
                PlusLettersItem(value) => result.plus_letters.push(value),
                DistanceItem(value) => result.distances.push(value),
                LateralityItem(value) => result.lateralities.push(value),
                CorrectionItem(value) => result.corrections.push(value),
                PinHoleItem(value) => result.pinholes.push(value),
                Text(_) => result.unhandled.push(item),
                Unhandled(_) => result.unhandled.push(item),
            }
        }
        result
    }
}