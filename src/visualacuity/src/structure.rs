use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::slice::Iter;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;

use crate::charts::ChartRow;
use crate::helpers::RoundPlaces;
use crate::DistanceUnits::NotProvided;
use crate::VisualAcuityError::*;
use crate::*;

lazy_static! {
    static ref PATTERN_FRACTION: Regex =
        Regex::new(r"^\s*(\d+(?:\.\d*)?)\s*/\s*(\d+(?:\.\d*)?)\s*$").expect("");
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Fraction(pub(crate) (f64, f64));

impl Deref for Fraction {
    type Target = (f64, f64);

    fn deref(&self) -> &Self::Target {
        return &self.0
    }
}

impl Display for Fraction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Fraction((num, den)) = self;
        write!(f, "{num}/{den}")
    }
}
impl Eq for Fraction {}
impl Hash for Fraction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        format!("{self:?}").hash(state)
    }
}

impl<T: Into<f64>> From<(T, T)> for Fraction {
    fn from((n, d): (T, T)) -> Self {
        Self((n.into(), d.into()))
    }
}

impl<T: From<f64>> From<Fraction> for (T, T) {
    fn from(fraction: Fraction) -> Self {
        (fraction.0 .0.into(), fraction.0 .1.into())
    }
}

impl FromStr for Fraction {
    type Err = VisualAcuityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = PATTERN_FRACTION
            .captures(s)
            .and_then(|c| Some((c.get(1)?.as_str(), c.get(2)?.as_str())))
            .map(|(n, d)| Ok((n.parse()?, d.parse()?)))
            .unwrap_or_else(|| Err(ParseError(format!("{s}"))));
        Ok(Self(value?))
    }
}

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub enum FixationPreference {
    CSM,
    CUSM,
    CSUM,
    CUSUM,
    UCSM,
    UCUSM,
    UCSUM,
    UCUSUM,
    // dunno:
    FixAndFollow,
    NoFixAndFollow,
    FixNoFollow,
    Prefers,
    Holds,
    Eccentric,
}

impl Display for FixationPreference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub enum NotTakenReason {
    NT,
    Unable,
    Refused,
    Sleeping,
    Prosthesis,
    SeeMR,
}

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub enum ParsedItem {
    SnellenFraction(String),
    Jaeger(String),
    Teller(String),
    ETDRS(String),
    NearTotalLoss(String, DistanceUnits),
    VisualResponse(String),
    CrossReferenceItem(String),
    PlusLettersItem(i32),
    NotTakenItem(NotTakenReason),

    // Visit Info
    DistanceItem(DistanceOfMeasurement),
    LateralityItem(Laterality),
    CorrectionItem(Correction),
    PinHoleItem(PinHole),

    Text(String), // text that isn't really part of a structured VA
    Unhandled(String),
}

impl Display for ParsedItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let formatted = match self {
            SnellenFraction(s)
            | Jaeger(s)
            | ETDRS(s)
            | Teller(s)
            | VisualResponse(s)
            | CrossReferenceItem(s) => s.to_string(),
            PlusLettersItem(n) => {
                if *n > 0 {
                    format!("+{self}")
                } else {
                    format!("{n}")
                }
            }
            NearTotalLoss(method, distance) => match distance.to_feet() {
                Ok(feet) => format!("{method} @ {} feet", feet.round_places(2)),
                _ => format!("{method}"),
            },
            NotTakenItem(reason) => format!("{reason:?}"),
            DistanceItem(d) => format!("{d}"),
            LateralityItem(l) => format!("{l}"),
            CorrectionItem(c) => format!("{c}"),
            PinHoleItem(effect) => format!("{effect:?}"),
            Text(_) | Unhandled(_) => format!("[text]"), // No PHI leaking here!
        };
        write!(f, "{formatted}")
    }
}

impl ParsedItem {
    pub(crate) fn find_chart_row(&self) -> VisualAcuityResult<&ChartRow> {
        let key = self.chart_row_key()?;
        match ChartRow::find(&key) {
            Some(chart_row) => Ok(chart_row),
            None => Err(ChartRowNotFound(key)),
        }
    }

    pub(crate) fn chart_row_key(&self) -> VisualAcuityResult<String> {
        match self {
            SnellenFraction(_) | ETDRS { .. } | Teller(_) | Jaeger(_) => Ok(self.to_string()),
            NearTotalLoss(s, ..) => Ok(s.to_string()),
            _ => Err(NoSnellenEquivalent(self.to_string())),
        }
    }

    pub(crate) fn measurement_distance(&self) -> &DistanceUnits {
        match self {
            NearTotalLoss(_, distance) => distance,
            _ => &NotProvided,
        }
    }
}

#[derive(PartialEq, Clone, Debug, Default)]
pub struct ParsedItemCollection(pub(crate) Vec<ParsedItem>);

impl_into_iter!(ParsedItemCollection, Vec<ParsedItem>);

impl ParsedItemCollection {
    pub fn iter(&self) -> Iter<'_, ParsedItem> {
        self.0.iter()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl FromIterator<ParsedItem> for ParsedItemCollection {
    fn from_iter<I: IntoIterator<Item = ParsedItem>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VAFormat {
    #[default]
    Unknown,
    Snellen,
    Jaeger,
    ETDRS,
    Teller,
    NearTotalLoss,
    VisualResponse,
    PinHole,
    Binocular,
    NotTaken,
    CrossReference,
}

impl From<ParsedItem> for VAFormat {
    fn from(value: ParsedItem) -> Self {
        match value {
            SnellenFraction { .. } => VAFormat::Snellen,
            Jaeger { .. } => VAFormat::Jaeger,
            Teller { .. } => VAFormat::Teller,
            ETDRS { .. } => VAFormat::ETDRS,
            NearTotalLoss { .. } => VAFormat::NearTotalLoss,
            VisualResponse { .. } => VAFormat::VisualResponse,
            PinHoleItem(_) => VAFormat::PinHole,
            NotTakenItem(_) => VAFormat::NotTaken,
            CrossReferenceItem(_) => VAFormat::CrossReference,
            _ => VAFormat::Unknown,
        }
    }
}

impl Display for VAFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
