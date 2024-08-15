extern crate visualacuity_proc_macro;

use std::fmt::{Display, Formatter};
use std::hash::Hash;

use visualacuity_proc_macro::DebugAsDisplay;

#[derive(Default, Clone, Debug, DebugAsDisplay, PartialEq, Eq, Hash)]
pub enum Laterality {
    #[default]
    Unknown,
    OS,
    OD,
    OU,
}

#[derive(Default, Clone, Debug, DebugAsDisplay, PartialEq, Eq, Hash)]
pub enum DistanceOfMeasurement {
    #[default]
    Unknown,
    Near,
    Distance,
}

#[derive(Default, Clone, Debug, DebugAsDisplay, PartialEq, Eq, Hash)]
pub enum Correction {
    #[default]
    Unknown,
    CC,
    SC,
    Manifest,
}

#[derive(Default, Clone, Debug, DebugAsDisplay, PartialEq, Eq, Hash)]
pub enum PinHole {
    #[default]
    Unknown,
    With,
    Without,
}
