use std::collections::BTreeMap;

use crate::dataquality::DataQuality;
use crate::errors::OptionResult;
use crate::logmar::{LogMarBase, LogMarPlusLetters};
use crate::parser::Content;
use crate::snellen_equivalent::SnellenEquivalent;
use crate::structure::{Fraction, VAFormat};
use crate::Correction::Manifest;
use crate::DataQuality::*;
use crate::VisualAcuityError::MultipleValues;
use crate::*;
use crate::{CorrectionItem, ParsedItem, ParsedItemCollection, VisualAcuityResult};
use itertools::Itertools;
use metadata::DistanceOfMeasurement::Distance;
use metadata::{Correction, DistanceOfMeasurement, Laterality, PinHole};

pub(crate) mod metadata;
mod tests;

#[derive(Default, PartialEq, Debug, Clone)]
pub struct EntryMetadata {
    /// The laterality (typically retrieved from the field name)
    pub laterality: Laterality,
    /// The distance of measurement (typically retrieved from the field name)
    pub distance_of_measurement: DistanceOfMeasurement,
    /// Whether correction was used (typically retrieved from the field name)
    pub correction: Correction,
    /// Whether the observation involved a pin-hole test (typically retrieved from field name)
    pub pinhole: PinHole,
}

impl EntryMetadata {
    pub(crate) fn with(mut self, item: ParsedItem) -> Self {
        use DistanceOfMeasurement::Unknown;
        if item == CorrectionItem(Manifest) && self.distance_of_measurement == Unknown {
            // Manifest implies Distance.
            self.distance_of_measurement = Distance;
        }

        // Incrementally build up the struct from each item
        match item {
            CorrectionItem(correction) => Self { correction, ..self },
            DistanceItem(distance_of_measurement) => Self {
                distance_of_measurement,
                ..self
            },
            LateralityItem(laterality) => Self { laterality, ..self },
            PinHoleItem(pinhole) => Self { pinhole, ..self },
            _ => self,
        }
    }
}

/// The public return type representing a parsed & processed set of EHR notes for a patient visit
#[derive(PartialEq, Debug, Clone)]
pub struct Visit(pub(crate) BTreeMap<String, Option<VisitNote>>);

impl_into_iter!(Visit, BTreeMap<String, Option<VisitNote>>);

/// The parsed & processed observations from an EHR field (with "plus" columns merged if possible)
#[derive(PartialEq, Debug, Clone)]
pub struct VisitNote {
    /// The contents of the original text field
    pub text: String,
    /// The contents of the associated "Plus" field, when available
    pub text_plus: String,

    /// Data quality label
    pub data_quality: DataQuality,

    /// The laterality (typically retrieved from the field name)
    pub laterality: Laterality,
    /// The distance of measurement (typically retrieved from the field name)
    pub distance_of_measurement: DistanceOfMeasurement,
    /// Whether correction was used (typically retrieved from the field name)
    pub correction: Correction,
    /// Whether the observation involved a pin-hole test (typically retrieved from field name)
    pub pinhole: PinHole,

    /// The detected format, e.g. Snellen or Teller
    pub va_format: VisualAcuityResult<VAFormat>,
    ///  When a patient reads a partial line, how many letters were indicated in the note  (+/-)
    pub plus_letters: Vec<i32>,

    /// The "normalized" text describing the visual acuity observation
    pub extracted_value: String,
    /// The Snellen equivalent of the visual acuity (if available), expressed as a fraction
    pub snellen_equivalent: OptionResult<Fraction>,
    /// The LogMAR equivalent of the visual acuity (if available), not considering partial lines
    pub log_mar_base: OptionResult<f64>,
    /// The LogMAR equivalent of the visual acuity (if available), with consideration of partial lines
    pub log_mar_base_plus_letters: OptionResult<f64>,
}

impl VisitNote {
    /// Given a variable length set of `ParsedItem`s, determine which items represent the visual
    /// acuity measurements etc. The task here mostly has to do with prioritization/disambiguation.
    pub(crate) fn new(
        entry_metadata: EntryMetadata,
        parsed_text: Content<ParsedItemCollection>,
        parsed_text_plus: Content<ParsedItemCollection>,
    ) -> VisualAcuityResult<Self> {
        let mut data_quality = parsed_text.data_quality.max(parsed_text_plus.data_quality);
        let Content {
            input: text,
            content: parsed_text,
            ..
        } = parsed_text;
        let Content {
            input: text_plus,
            content: parsed_text_plus,
            ..
        } = parsed_text_plus;
        let parsed_notes = ParsedItemCollection(
            [parsed_text, parsed_text_plus]
                .into_iter()
                .flatten()
                .collect(),
        );
        let sifted = &SiftedParsedItems::sift(parsed_notes);
        let base_acuity = &sifted.base_acuity;
        let log_mar_base = base_acuity.clone().then(|v| v.log_mar_base());
        let log_mar_base_plus_letters = base_acuity
            .clone()
            .then(|v| v.log_mar_plus_letters(&sifted.plus_letters));
        let other_options = &sifted
            .acuities
            .iter()
            .chain(&sifted.other_observations)
            .cloned()
            .collect_vec();
        let va_format = get_va_format(&base_acuity, other_options);
        let extracted_value = extract_value(&base_acuity, &sifted.other_observations);
        let snellen_equivalent = base_acuity.clone().then(|v| v.snellen_equivalent());
        let plus_letters = sifted.plus_letters.clone();

        data_quality = match base_acuity {
            OptionResult::None => NoValue,
            OptionResult::Err(MultipleValues(_)) => Multiple,
            _ => data_quality,
        };
        if plus_letters.len() > 1 && data_quality == Exact {
            // "20/40 +3 -2" => not exact. Is there a cleaner way to do this?
            data_quality = ConvertibleFuzzy;
        }

        let EntryMetadata {
            correction,
            pinhole,
            laterality,
            distance_of_measurement,
        } = entry_metadata;

        Ok(VisitNote {
            text: text.to_string(),
            text_plus: text_plus.to_string(),
            extracted_value,
            data_quality,
            distance_of_measurement,
            correction,
            pinhole,
            laterality,
            plus_letters,
            va_format,
            snellen_equivalent,
            log_mar_base,
            log_mar_base_plus_letters,
        })
    }
}

/// Retrieve the "normalized" text representing the primary observation in a given EHR note.
fn extract_value(item: &OptionResult<ParsedItem>, other_observations: &Vec<ParsedItem>) -> String {
    match item {
        OptionResult::Some(s) => s.to_string(),
        OptionResult::Err(_) => format!("Error"),
        OptionResult::None => match other_observations.first() {
            Some(s) => s.to_string(),
            None => String::default(),
        },
    }
}

/// Determine the format/method of the VA text.
fn get_va_format(
    base_acuity: &OptionResult<ParsedItem>,
    other_acuities: &Vec<ParsedItem>,
) -> VisualAcuityResult<VAFormat> {
    // If there's a base_acuity, use the associated format.
    // Otherwise, see if there's a single format in the other_acuities.
    match base_acuity {
        OptionResult::Some(ref value) => Ok(value.clone().into()),
        _ => match other_acuities
            .into_iter()
            .cloned()
            .map_into()
            .unique()
            .at_most_one()
        {
            Ok(Some(item)) => Ok(item),
            Ok(None) => Ok(Default::default()),
            Err(e) => Err(e.into()),
        },
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
    pin_hole: Vec<PinHole>,
    unhandled: Vec<ParsedItem>,
    base_acuity: OptionResult<ParsedItem>,
}

impl SiftedParsedItems {
    /// Iterates through parsed items, assigning each variant of `ParsedItem` into a bin/category.
    fn sift(parsed_notes: ParsedItemCollection) -> Self {
        let mut result = Self::default();
        for item in parsed_notes {
            match item {
                SnellenFraction { .. }
                | Jaeger { .. }
                | Teller { .. }
                | ETDRS { .. }
                | VisualResponse { .. }
                | CrossReferenceItem(_)
                | NearTotalLoss { .. } => result.acuities.push(item),

                NotTakenItem(_) => result.other_observations.push(item),

                PlusLettersItem(value) => result.plus_letters.push(value),
                DistanceItem(value) => result.distances.push(value),
                LateralityItem(value) => result.lateralities.push(value),
                CorrectionItem(value) => result.corrections.push(value),
                PinHoleItem(value) => result.pin_hole.push(value),

                Text(_) => result.unhandled.push(item),
                Unhandled(_) => result.unhandled.push(item),
            }
        }
        result.base_acuity = Self::base_acuity_(&result);
        result
    }

    /// Given `ParsedItem`s, determine which one reperesents a "base acuity." If none are present,
    /// consider "other observations" (e.g. binocular fixation) that might be the primary observation.
    fn base_acuity_(&self) -> OptionResult<ParsedItem> {
        let unique_acuities = self
            .acuities
            .iter()
            .rev() // Take the *last* equivalent thing (e.g. ETDRS)
            .unique_by(|&acuity| acuity.snellen_equivalent())
            .collect_vec();

        let acuity_item = unique_acuities.into_iter().rev().at_most_one();

        match acuity_item {
            // If no acuity is present, see if we have another kind of observation
            Ok(Some(VisualResponse(v))) => OptionResult::Some(VisualResponse(v.clone())),
            Ok(Some(v)) => OptionResult::Some(v.clone()),
            Err(e) => OptionResult::Err(e.into()),
            Ok(None) => OptionResult::None,
        }
    }
}
