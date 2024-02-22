use std::hash::{Hash, Hasher};
use std::str::FromStr;
use lazy_static::lazy_static;
use regex::Regex;
use crate::{VisualAcuityError, VisualAcuityResult};
use crate::DistanceUnits::{Centimeters, Feet};
use crate::VisualAcuityError::ParseError;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum DistanceUnits {
    #[default]
    NotProvided,
    Unhandled(String),
    Feet(f64),
    Inches(f64),
    Meters(f64),
    Centimeters(f64),
    FeetRange((f64, f64)),
    InchesRange((f64, f64)),
    MetersRange((f64, f64)),
    CentimetersRange((f64, f64)),
}

impl Hash for DistanceUnits {
    fn hash<H: Hasher>(&self, state: &mut H) {
        format!("{self:?}").hash(state)
    }
}

impl Eq for DistanceUnits { }


impl DistanceUnits {
    pub(crate) fn to_feet(&self) -> VisualAcuityResult<f64> {
        use DistanceUnits::*;
        use crate::VisualAcuityError::DistanceConversionError;
        fn average (low: f64, high: f64) -> f64 { 0.5 * (low + high) }
        match *self {
            Feet(feet) => Ok(feet),
            Inches(inches) => Ok(inches / 12.0),
            Meters(meters) => Ok(meters * 3.28084),
            Centimeters(cm) => Ok(cm * 3.28084e-2),
            FeetRange((low, high)) => Feet(average(low, high)).to_feet(),
            InchesRange((low, high)) => Inches(average(low, high)).to_feet(),
            MetersRange((low, high)) => Meters(average(low, high)).to_feet(),
            CentimetersRange((low, high)) => Centimeters(average(low, high)).to_feet(),
            _ => Err(DistanceConversionError)
        }
    }
}

lazy_static! {
static ref PATTERN_DISTANCE: Regex = Regex::new(r"^\s*(\d+(?:\.\d*)?)\s*(cm|ft).*$").expect("");
}

impl FromStr for DistanceUnits {
    type Err = VisualAcuityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Simplified distance parsing for the chart config files
        let e = || Err(ParseError(s.to_string()));
        PATTERN_DISTANCE.captures(s)
            .and_then(|c| Some((c.get(1)?.as_str(), c.get(2)?.as_str())))
            .map_or_else(|| e(), |(number, units)| match (number.parse(), units) {
                (Ok(n), "cm") => Ok(Centimeters(n)),
                (Ok(n), "ft") => Ok(Feet(n)),
                _ => e()
            })

    }
}