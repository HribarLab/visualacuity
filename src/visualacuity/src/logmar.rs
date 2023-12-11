use crate::{SnellenRow, ParsedItem, VisualAcuityResult, ParsedItemCollection};
use crate::ParsedItem::*;
use crate::VisualAcuityError::*;
use crate::snellen_equivalent::SnellenEquivalent;

pub(crate) trait LogMarBase {
    fn log_mar_base(&self) -> VisualAcuityResult<f64>;
}

impl LogMarBase for ParsedItem<'_> {
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

#[derive(PartialEq)]
pub struct LogMarEstimate<'a> {
    pub parsed_items: ParsedItemCollection<'a>,
    pub base_item: ParsedItem<'a>,
    pub plus_letters: Vec<i32>,
    pub log_mar_base: f64,
    pub log_mar: f64,
}

impl<'a> LogMarEstimate<'a> {
    pub fn from_va(
        parsed_items: ParsedItemCollection<'a>,
        parsed_plus: ParsedItemCollection<'a>
    ) -> VisualAcuityResult<Self> {
        let combined = ParsedItemCollection(parsed_items.into_iter().chain(parsed_plus.into_iter()).collect());
        let base_item = combined.base_acuity()?;
        let plus_letters = combined.plus_letters();
        let log_mar_base = base_item.log_mar_base()?;
        let log_mar = base_item.log_mar_plus_letters(&plus_letters)?;
        Ok(Self { parsed_items: combined, base_item, plus_letters, log_mar_base, log_mar })
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

impl LogMarPlusLetters for ParsedItem<'_> {
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

#[cfg(test)]
mod tests {
    use crate::JaegerRow::*;
    use crate::SnellenRow::*;
    use super::*;

    #[test]
    fn test_logmar_estimate() {
        let test_cases = [
            (
                vec![
                    Snellen(S20),
                    PlusLettersItem(1),
                    PlusLettersItem(2),
                    PlusLettersItem(3),
                ],
                Ok((Snellen(S20), vec![1, 2, 3])),
            ),
            (
                vec![
                    PlusLettersItem(1),
                    PlusLettersItem(2),
                    PlusLettersItem(3),
                ],
                Err(NoValue),
            ),
            (
                vec![
                    Jaeger(J1),
                    PlusLettersItem(1),
                    PlusLettersItem(2),
                    PlusLettersItem(3),
                    Snellen(S20),
                ],
                Err(MultipleValues(format!("[Jaeger(J1), Snellen(S20)]"))),
            ),
        ];

        for (parsed_items, expected) in test_cases {
            let parsed_items = ParsedItemCollection(parsed_items);
            let parsed_plus = ParsedItemCollection(vec![]);
            let actual = match LogMarEstimate::from_va(parsed_items, parsed_plus) {
                Ok(LogMarEstimate { base_item, plus_letters, .. }) => Ok((base_item, plus_letters)),
                Err(e) => Err(e)
            };
            assert_eq!(actual, expected);
        }
    }
}