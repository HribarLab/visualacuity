use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::slice::Iter;
use std::str::FromStr;
use itertools::{ExactlyOneError, Itertools};
use derive_more::{Deref, IntoIterator};
use lazy_static::lazy_static;
use regex::Regex;
use crate::{DistanceUnits, VisualAcuityError, VisualAcuityResult};
use crate::ParsedItem::*;
use crate::VisualAcuityError::*;
use crate::charts::ChartRow;
use crate::DistanceUnits::NotProvided;

lazy_static! {
static ref PATTERN_FRACTION: Regex = Regex::new(r"^\s*(\d+(?:\.\d*)?)\s*/\s*(\d+(?:\.\d*)?)\s*$").expect("");
}

pub trait TInput: PartialEq + Debug + Clone {}
impl<T> TInput for T where T: PartialEq + Debug + Clone {}


#[derive(PartialEq, Debug)]
pub struct Input<'input, T: TInput> {
    pub content: T,
    pub left: usize,
    pub right: usize,
    pub input: &'input str,
}

impl<'input, T: TInput> Deref for Input<'input, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<'input, T: TInput> PartialEq<T> for Input<'input, T> {
    fn eq(&self, other: &T) -> bool {
        self.content == *other
    }
}

impl<'input, T: TInput> Display for Input<'input, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.input[self.left..self.right])
    }
}

#[derive(Clone, Copy, PartialEq, Deref, Debug)]
pub struct Fraction(pub(crate) (f64, f64));

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
    fn from((n, d): (T, T)) -> Self { Self((n.into(), d.into())) }
}

impl From<Fraction> for (f64, f64) {
    fn from(value: Fraction) -> Self {
        *value
    }
}

impl FromStr for Fraction {
    type Err = VisualAcuityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = PATTERN_FRACTION.captures(s)
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
pub enum PinHoleEffect {
    NI,
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
    TellerCard(String),
    TellerCyCm(String),
    ETDRS(String),
    LowVision(String, DistanceUnits),
    PinHoleEffectItem(PinHoleEffect),
    BinocularFixation(FixationPreference),
    PlusLettersItem(i32),
    NotTakenItem(NotTakenReason),

    // Visit Info
    DistanceItem(DistanceOfMeasurement),
    LateralityItem(Laterality),
    CorrectionItem(Correction),
    PinHoleItem(PinHole),

    // Comment { text: String },
    Text(String),  // text that isn't really part of a structured VA
    Unhandled(String),
}

impl Display for ParsedItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let formatted = match self {
            SnellenFraction(s)
            | Jaeger(s)
            | ETDRS(s)
            | TellerCard(s)
            | TellerCyCm(s) => {
                s.to_string()
            },
            PlusLettersItem(n) => if *n > 0 { format!("+{self}") } else { format!("{n}") },
            PinHoleItem(effect) => format!("{effect:?}"),
            LowVision(method, distance) => match distance.to_feet() {
                Ok(feet) => format!("{method} @ {feet:?} feet"),
                _ => format!("{method}")
            },
            PinHoleEffectItem(effect) => format!("{effect:?}"),
            BinocularFixation(preference) => format!("{preference:?}"),
            NotTakenItem(reason) => format!("{reason:?}"),
            DistanceItem(d) => format!("{d}"),
            LateralityItem(l) => format!("{l}"),
            CorrectionItem(c) => format!("{c}"),
            Text(_) | Unhandled(_) => format!("[text]"), // No PHI leaking here!
        };
        write!(f, "{formatted}")
    }
}

impl ParsedItem {
    pub(crate) fn is_base(&self) -> bool {
        match self {
            &SnellenFraction { .. }
                | &Jaeger { .. }
                // | &TellerCard { .. }
                | &ETDRS { .. }
                | &LowVision { .. } => true,
            _ => false,
        }
    }

    pub(crate) fn find_chart_row(&self) -> VisualAcuityResult<&ChartRow> {
        let key = self.chart_row_key()?;
        match ChartRow::find(&key) {
            Some(chart_row) => Ok(chart_row),
            None => Err(ChartRowNotFound(key)),
        }
    }

    pub(crate) fn chart_row_key(&self) -> VisualAcuityResult<String> {
        match self {
            SnellenFraction(_)
            | ETDRS { .. }
            | TellerCard(_)
            | TellerCyCm(_)
            | Jaeger(_) => {
                Ok(self.to_string())
            },
            LowVision(method,  ..) => {
                Ok(method.to_string())
            },
            _ => Err(NoSnellenEquivalent(self.to_string()))
        }
    }

    pub(crate) fn measurement_distance(&self) -> &DistanceUnits {
        match self {
            LowVision(_, distance) => distance,
            _ => &NotProvided
        }

    }
}

#[derive(IntoIterator, PartialEq, Clone, Debug, Default)]
pub struct ParsedItemCollection(Vec<ParsedItem>);

impl ParsedItemCollection {

    pub fn iter(&self) -> Iter<'_, ParsedItem> { self.0.iter() }
    pub fn len(&self) -> usize { self.0.len() }

    fn get_one<T, F>(&self, f: F) -> VisualAcuityResult<T>
        where T: Clone + Debug, F: FnMut(&ParsedItem) -> Option<T>
    {
        Ok(self.iter().filter_map(f).exactly_one()?.into())
    }

    pub fn laterality(&self) -> VisualAcuityResult<Laterality> {
        self.get_one(|item| match item {
            LateralityItem(l) => Some(l.clone()),
            _ => None
        })
    }

    pub fn distance_of_measurement(&self) -> VisualAcuityResult<DistanceOfMeasurement> {
        self.get_one(|item| match item {
            DistanceItem(d) => Some(d.clone()),
            _ => None
        })
    }

    pub fn correction(&self) -> VisualAcuityResult<Correction> {
        self.get_one(|item| match item {
            CorrectionItem(c) => Some(c.clone()),
            _ => None
        })
    }

    pub fn plus_letters(&self) -> Vec<i32> {
        self.iter().filter_map(|item| match item {
            PlusLettersItem(value) => Some(*value),
            _ => None
        }).collect()
    }

    pub fn base_acuity(&self) -> VisualAcuityResult<ParsedItem> {
        fn match_base_item<'b>(item: &ParsedItem) -> Option<ParsedItem> {
            if item.is_base() { Some(item.clone()) } else { None }
        }
        self.get_one(match_base_item)
    }

    pub fn method(&self) -> Method {
        self.get_one(|item| match item {
            SnellenFraction { .. } => Some(Method::Snellen),
            Jaeger { .. } => Some(Method::Jaeger),
            ETDRS { .. } => Some(Method::ETDRS),
            TellerCard { .. } => Some(Method::Teller),
            TellerCyCm { .. } => Some(Method::Teller),
            LowVision { .. } => Some(Method::LowVision),
            _ => None
        }).unwrap_or(Method::Unknown)
    }
}

impl FromIterator<ParsedItem> for ParsedItemCollection {
    fn from_iter<I: IntoIterator<Item=ParsedItem>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Laterality {
    Error(VisualAcuityError),
    #[default]
    Unknown,
    OS,
    OD,
    OU
}

impl Display for Laterality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DistanceOfMeasurement {
    Error(VisualAcuityError),
    #[default]
    Unknown,
    Near,
    Distance,
}

impl Display for DistanceOfMeasurement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub(crate) trait Disambiguate where Self: Default + Clone + Eq + Hash  {
    fn on_conflict<I: Iterator<Item=Self>>(e: ExactlyOneError<I>) -> Self;

    fn disambiguate<I: IntoIterator<Item=Self> + Clone>(iter: &I) -> Self {
        match iter.clone().into_iter().unique().at_most_one() {
            Ok(Some(item)) => item,
            Ok(None) => Self::default(),
            Err(e) => Self::on_conflict(e)
        }
    }
}

impl Disambiguate for DistanceOfMeasurement {
    fn on_conflict<I: Iterator<Item=Self>>(e: ExactlyOneError<I>) -> Self {
        Self::Error(e.into())
    }
}

impl Disambiguate for Laterality {
    fn on_conflict<I: Iterator<Item=Self>>(e: ExactlyOneError<I>) -> Self {
        Self::Error(e.into())
    }
}
impl Disambiguate for Correction {
    fn on_conflict<I: Iterator<Item=Self>>(e: ExactlyOneError<I>) -> Self {
        Self::Error(e.into())
    }
}

impl Disambiguate for Method {
    fn on_conflict<I: Iterator<Item=Self>>(e: ExactlyOneError<I>) -> Self {
        Self::Error(e.into())
    }
}

impl Disambiguate for PinHole {
    fn on_conflict<I: Iterator<Item=Self>>(e: ExactlyOneError<I>) -> Self {
        Self::Error(e.into())
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Correction {
    Error(VisualAcuityError),
    #[default]
    Unknown,
    CC,
    SC,
}

impl Display for Correction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PinHole {
    Error(VisualAcuityError),
    #[default]
    Unknown,
    With,
    Without,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Method {
    Error(VisualAcuityError),
    #[default]
    Unknown,
    Snellen,
    Jaeger,
    ETDRS,
    Teller,
    LowVision,
    PinHole,
    Binocular,
    NotTaken,
}

impl From<ParsedItem> for Method {
    fn from(value: ParsedItem) -> Self {
        match value {
            SnellenFraction { .. } => Method::Snellen,
            Jaeger { .. } => Method::Jaeger,
            TellerCard { .. } => Method::Teller,
            TellerCyCm { .. } => Method::Teller,
            ETDRS { .. } => Method::ETDRS,
            LowVision { .. } => Method::LowVision,
            PinHoleEffectItem(_) => Method::PinHole,
            BinocularFixation(_) => Method::Binocular,
            NotTakenItem(_) => Method::NotTaken,
            _ => Method::Unknown,
        }
    }
}
