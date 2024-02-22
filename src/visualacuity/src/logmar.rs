use crate::{ParsedItem, VisualAcuityResult};
use crate::ParsedItem::*;
use crate::VisualAcuityError::*;
use crate::snellen_equivalent::SnellenEquivalent;

pub(crate) trait LogMarBase {
    fn log_mar_base(&self) -> VisualAcuityResult<f64>;
}

impl LogMarBase for ParsedItem {
    fn log_mar_base(&self) -> VisualAcuityResult<f64> {
        fn round_digits(x: f64, n: u32) -> f64 {
            let magnitude = 10.0_f64.powf(n as f64);
            (magnitude * x).round() / magnitude
        }

        fn negative_log(top: f64, bottom: f64) -> f64 {
            if top == bottom { 0.0 } else { -(top / bottom).log10() }
        }

        let (distance, row) = *self.snellen_equivalent().map_err(|_| NotImplemented)?;
        match &self {
            ETDRS { .. } => {
                // round these ones to one decimal place?
                // https://www.researchgate.net/figure/Conversions-Between-Letter-LogMAR-and-Snellen-Visual-Acuity-Scores_tbl1_258819613
                let exact = negative_log(distance, row);
                Ok(round_digits(exact, 1))
            }
            _ => Ok(negative_log(distance, row))
        }
    }
}

pub trait LogMarPlusLetters {
    fn log_mar_plus_letters(&self, plus_letters: &Vec<i32>) -> VisualAcuityResult<f64>;
}

impl LogMarPlusLetters for ParsedItem {
    fn log_mar_plus_letters(&self, plus_letters: &Vec<i32>) -> VisualAcuityResult<f64> {
        if plus_letters.len() == 0 {
            self.log_mar_base()
        }
        else {
            let row = self.find_chart_row().map_err(|row| ChartRowNotFound(row.to_string()))?;
            let mut result = self.log_mar_base()?;
            for &pl in plus_letters {
                result += pl as f64 * row.log_mar_increment(pl)?;
            }
            Ok(result)
        }
    }
}

