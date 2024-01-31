use crate::{SnellenRow, ParsedItem, VisualAcuityResult};
use crate::ParsedItem::*;
use crate::VisualAcuityError::*;
use crate::snellen_equivalent::SnellenEquivalent;

pub(crate) trait LogMarBase {
    fn log_mar_base(&self) -> VisualAcuityResult<f64>;
}

impl LogMarBase for ParsedItem {
    fn log_mar_base(&self) -> VisualAcuityResult<f64> {
        match &self {
            ETDRS { .. } | LowVision { .. } => {
                // round these ones to one decimal place?
                // https://www.researchgate.net/figure/Conversions-Between-Letter-LogMAR-and-Snellen-Visual-Acuity-Scores_tbl1_258819613
                let (distance, row) = self.snellen_equivalent()
                    .map_err(|_| LogMarNotImplemented)?;
                let exact = negative_log(distance, row);
                Ok(round_digits(exact, 1))
            }
            _ => {
                let (distance, row) = self.snellen_equivalent()
                    .map_err(|_| LogMarNotImplemented)?;
                Ok(negative_log(distance, row))
            }
        }
    }
}

fn round_digits(x: f64, n: u32) -> f64 {
    let magnitude = 10.0_f64.powf(n as f64);
    (magnitude * x).round() / magnitude
}

impl LogMarBase for SnellenRow {
    fn log_mar_base(&self) -> VisualAcuityResult<f64> {
        Ok(negative_log(20, *self as u16))
    }
}

fn negative_log(top: u16, bottom: u16) -> f64 {
    let log = (top as f64 / bottom as f64).log10();
    if log == 0.0 { 0.0 } else { -log }
}

fn log_mar_increment(base: &SnellenRow, plus_letters: i32) -> VisualAcuityResult<f64> {
    // Shortcut:
    // -log(20/a) - -log(20/b)
    // = log(a/b)
    let get_from_to = || {
        if plus_letters > 0 { Ok((*base, base.positive()?)) }
        else if plus_letters < 0 { Ok((base.negative()?, *base)) }
        else { Err(LogMarInvalidPlusLetters(format!("{plus_letters}"))) }
    };
    let (from, to) =  get_from_to()?;
    Ok(negative_log(from as u16, to as u16) / to.n_items()? as f64)
}

pub trait LogMarPlusLetters {
    fn log_mar_plus_letters(&self, plus_letters: &Vec<i32>) -> VisualAcuityResult<f64>;
}

impl LogMarPlusLetters for ParsedItem {
    fn log_mar_plus_letters(&self, plus_letters: &Vec<i32>) -> VisualAcuityResult<f64> {
        match (plus_letters.len(), self) {
            (_, Snellen(row)) => row.log_mar_plus_letters(plus_letters),
            (0, _) => self.log_mar_base(),
            (_, _) => Err(LogMarNotImplemented)
        }
    }
}

impl LogMarPlusLetters for SnellenRow {
    fn log_mar_plus_letters(&self, plus_letters: &Vec<i32>) -> VisualAcuityResult<f64> {
        let mut result = self.log_mar_base()?;
        for &pl in plus_letters {
            result += pl as f64 * log_mar_increment(self, pl)?;
        }
        Ok(result)
    }
}
