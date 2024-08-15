use crate::charts::ChartRow;
use crate::snellen_equivalent::SnellenEquivalent;
use crate::VisualAcuityError::*;
use crate::{DistanceUnits, Fraction, ParsedItem, VisualAcuityResult};

pub(crate) trait LogMarBase {
    fn log_mar_base(&self) -> VisualAcuityResult<f64>;
}

impl LogMarBase for ParsedItem {
    fn log_mar_base(&self) -> VisualAcuityResult<f64> {
        use DistanceUnits::*;
        let row = self.find_chart_row()?;
        let log_mar_base = row.log_mar_base()?;

        let meas_dist = self.measurement_distance();
        let ref_dist = &row.reference_distance;

        // Do we need to scale to a measured distance?
        match (meas_dist, ref_dist) {
            // No
            (NotProvided, NotProvided) => Ok(log_mar_base),
            // Yes
            _ => {
                let Ok(m) = meas_dist.to_feet() else {
                    return Err(NotImplemented);
                };
                let Ok(r) = ref_dist.to_feet() else {
                    return Err(NotImplemented);
                };
                Ok(log_mar_base - (m / r).log10())
            }
        }
    }
}

impl LogMarBase for ChartRow {
    fn log_mar_base(&self) -> VisualAcuityResult<f64> {
        match self.log_mar {
            Some(log_mar) => Ok(log_mar),
            _ => self
                .snellen_equivalent()
                .and_then(|fraction| fraction.log_mar_base())
                .map_err(|_| NotImplemented),
        }
    }
}

impl LogMarBase for Fraction {
    fn log_mar_base(&self) -> VisualAcuityResult<f64> {
        fn negative_log(top: f64, bottom: f64) -> f64 {
            if top == bottom {
                0.0
            } else {
                -(top / bottom).log10()
            }
        }

        let Fraction((distance, row)) = *self;
        Ok(negative_log(distance, row))
    }
}

pub trait LogMarPlusLetters {
    fn log_mar_plus_letters(&self, plus_letters: &Vec<i32>) -> VisualAcuityResult<f64>;
}

impl LogMarPlusLetters for ParsedItem {
    fn log_mar_plus_letters(&self, plus_letters: &Vec<i32>) -> VisualAcuityResult<f64> {
        if plus_letters.len() == 0 {
            return self.log_mar_base();
        }
        self.find_chart_row()
            .and_then(|row| row.log_mar_plus_letters(plus_letters))
            .map_err(|row| ChartRowNotFound(row.to_string()))
    }
}

impl LogMarPlusLetters for ChartRow {
    fn log_mar_plus_letters(&self, plus_letters: &Vec<i32>) -> VisualAcuityResult<f64> {
        macro_rules! err {
            ($s:expr) => {
                LogMarInvalidSnellenRow(format!($s))
            };
        }

        if plus_letters.len() == 0 {
            return self.log_mar_base();
        }
        fn pos(row: &ChartRow) -> Option<f64> {
            Some((row.next_log_mar? - row.log_mar?) / row.next_n_letters? as f64)
        }

        fn neg(row: &ChartRow) -> Option<f64> {
            Some((row.log_mar? - row.prev_log_mar?) / row.n_letters? as f64)
        }

        let mut result = self.log_mar_base()?;
        for &pl in plus_letters {
            let increment = if pl.is_positive() {
                pos(self).ok_or_else(|| err!("Missing next row values: {self:?}"))
            } else {
                neg(self).ok_or_else(|| err!("Missing previous row values: {self:?}"))
            };
            result += pl as f64 * increment?;
        }
        Ok(result)
    }
}
