use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::Deref;
use std::slice::Iter;
use std::str::FromStr;
use itertools::{ExactlyOneError, Itertools};
use derive_more::IntoIterator;
use crate::{VisualAcuityError, VisualAcuityResult};
use crate::ParsedItem::*;
use crate::VisualAcuityError::*;
use crate::SnellenRow::*;
use num_traits::FromPrimitive;

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

#[derive(Hash, Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
#[repr(u16)]
pub enum SnellenRow {
    S15 = 15,
    S20 = 20,
    S25 = 25,
    S30 = 30,
    S40 = 40,
    S50 = 50,
    S60 = 60,
    S70 = 70,
    S80 = 80,
    S100 = 100,
    S125 = 125,
    S150 = 150,
    S200 = 200,
    S250 = 250,
    S300 = 300,
    S400 = 400,
    S500 = 500,
    S600 = 600,

    // extended:
    S800 = 800,
    S640 = 640,
    S320 = 320,
    S160 = 160,
    S63 = 63,
    S32 = 32,
    S12 = 12,
    S10 = 10,
}

impl SnellenRow {
    pub(crate) fn n_items(&self) -> VisualAcuityResult<u16> {
        match self {
            S15 | S20 | S25 => Ok(6),
            S30 | S40 | S50 => Ok(5),
            S60 | S70 => Ok(4),
            S80 => Ok(3),
            S100 | S125 | S150 => Ok(2),
            S200 | S250 | S300 | S400 | S500 | S600 => Ok(1),
            _ => Err(LogMarInvalidSnellenRow(format!("{}", *self as u16))),
        }
    }

    pub(crate) fn positive(&self) -> VisualAcuityResult<Self> {
        match self {
            S15 => Ok(S15),
            S20 => Ok(S15),
            S25 => Ok(S20),
            S30 => Ok(S25),
            S40 => Ok(S30),
            S50 => Ok(S40),
            S60 => Ok(S50),
            S70 => Ok(S60),
            S80 => Ok(S70),
            S100 => Ok(S80),
            S125 => Ok(S100),
            S150 => Ok(S125),
            S200 => Ok(S150),
            S250 => Ok(S200),
            S300 => Ok(S250),
            S400 => Ok(S300),
            S500 => Ok(S400),
            S600 => Ok(S500),
            _ => Err(LogMarInvalidPlusLetters(format!("{} +n", *self as u16)))
        }
    }

    pub(crate) fn negative(&self) -> VisualAcuityResult<Self> {
        match self {
            &S15 => Ok(S20),
            &S20 => Ok(S25),
            &S25 => Ok(S30),
            &S30 => Ok(S40),
            &S40 => Ok(S50),
            &S50 => Ok(S60),
            &S60 => Ok(S70),
            &S70 => Ok(S80),
            &S80 => Ok(S100),
            &S100 => Ok(S125),
            &S125 => Ok(S150),
            &S150 => Ok(S200),
            &S200 => Ok(S250),
            &S250 => Ok(S300),
            &S300 => Ok(S400),
            &S400 => Ok(S500),
            &S500 => Ok(S600),
            &S600 => Ok(S600),
            _ => Err(LogMarInvalidPlusLetters(format!("{} -n", *self as u16)))
        }
    }
}

impl FromStr for SnellenRow {
    type Err = VisualAcuityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n = s.parse::<u16>().map_err(|e| ParseError(format!("{e:?}")))?;
        FromPrimitive::from_u16(n).ok_or(ParseError(format!("FromStr error")))
    }
}


#[derive(Hash, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum JaegerRow {
    J1PLUS = 0,
    J1 = 1,
    J2 = 2,
    J3 = 3,
    J4 = 4,
    J5 = 5,
    J6 = 6,
    J7 = 7,
    J8 = 8,
    J9 = 9,
    J10 = 10,
    J11 = 11,
    J12 = 12,
    J13 = 13,
    J14 = 14,
    J15 = 15,
    J16 = 16,
    J17 = 17,
    J18 = 18,
    J19 = 19,
    J20 = 20,
    J21 = 21,
    J22 = 22,
    J23 = 23,
    J24 = 24,
    J25 = 25,
    J26 = 26,
    J27 = 27,
    J28 = 28,
    J29 = 29,
}

impl FromStr for JaegerRow {
    type Err = VisualAcuityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim_start_matches(&['J', 'j']) {
            "1+" => Ok(JaegerRow::J1PLUS),
            s => Self::from_u16(s.parse()?).ok_or(ParseError(s.to_string()))
        }
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
pub enum LowVisionMethod {
    CountingFingers,
    HandMovement,
    LightPerception,
    NoLightPerception,
}

impl Display for LowVisionMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            LowVisionMethod::CountingFingers =>write!(f, "CF"),
            LowVisionMethod::HandMovement => write!(f, "HM"),
            LowVisionMethod::LightPerception => write!(f, "LP"),
            LowVisionMethod::NoLightPerception => write!(f, "NLP"),
        }
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
pub enum ParsedItem<'input> {
    Snellen(SnellenRow),
    Jaeger(JaegerRow),
    Teller { row: u16, card: u16 },
    ETDRS { letters: u32 },
    LowVision { method: LowVisionMethod, distance: Option<DistanceOfMeasurement> },
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
    Text(&'input str),  // text that isn't really part of a structured VA
    Unhandled(String),
}

impl ParsedItem<'_> {
    pub(crate) fn is_base(&self) -> bool {
        match self {
            &Snellen { .. }
                | &Jaeger { .. }
                | &Teller { .. }
                | &ETDRS { .. }
                | &LowVision { .. } => true,
            _ => false,
        }
    }
}

#[derive(IntoIterator, PartialEq, Clone, Debug)]
pub struct ParsedItemCollection<'a>(pub(crate) Vec<ParsedItem<'a>>);

impl<'a> ParsedItemCollection<'a> {
    pub fn iter(&self) -> Iter<'_, ParsedItem<'a>> {
        self.0.iter()
    }

    pub fn len(&self) -> usize { self.0.len() }

    fn get_one<T, F>(&self, f: F) -> VisualAcuityResult<T>
        where T: Clone + Debug, F: FnMut(&ParsedItem<'a>) -> Option<T>
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

    pub fn base_acuity(&self) -> VisualAcuityResult<ParsedItem<'a>> {
        fn match_base_item<'b>(item: &ParsedItem<'b>) -> Option<ParsedItem<'b>> {
            if item.is_base() { Some(item.clone()) } else { None }
        }
        self.get_one(match_base_item)
    }

    pub fn method(&self) -> Method {
        self.get_one(|item| match item {
            Snellen{ .. } => Some(Method::Snellen),
            Jaeger { .. } => Some(Method::Jaeger),
            ETDRS { .. } => Some(Method::ETDRS),
            Teller { .. } => Some(Method::Teller),
            LowVision { .. } => Some(Method::LowVision),
            _ => None
        }).unwrap_or(Method::Unknown)
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

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DistanceOfMeasurement {
    Error(VisualAcuityError),
    #[default]
    Unknown,
    Near,
    Distance,
}

pub(crate) trait Disambiguate where Self: Default + Clone + Eq + Hash  {
    fn on_conflict<I: Iterator<Item=Self>>(e: ExactlyOneError<I>) -> Self;

    fn disambiguate<I: IntoIterator<Item=Self>>(iter: I) -> Self {
        match iter.into_iter().unique().at_most_one() {
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
