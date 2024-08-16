use crate::charts::ChartRow;
use crate::VisualAcuityError::*;
use crate::*;

pub(crate) trait SnellenEquivalent {
    fn snellen_equivalent(&self) -> VisualAcuityResult<Fraction>;
}

impl SnellenEquivalent for ParsedItem {
    fn snellen_equivalent(&self) -> VisualAcuityResult<Fraction> {
        // This leans on data found in the files assets/charts/*.tsv
        let error = |_| NoSnellenEquivalent(self.to_string());
        match self.find_chart_row() {
            Ok(ChartRow {
                fraction: Some(ref_acuity),
                reference_distance,
                ..
            }) => {
                if reference_distance == &DistanceUnits::NotProvided {
                    // Found a chart row + no conversion necessary.
                    return Ok(ref_acuity.clone());
                }
                // Found a chart row + conversion necessary.
                let feet = self.measurement_distance().to_feet().map_err(error)?;
                let ref_feet = reference_distance.to_feet().map_err(error)?;
                let Fraction((converted_distance, ref_row)) = ref_acuity.clone();
                let converted_row = ref_row * ref_feet / feet;
                let rounded_row = (converted_row as u64) as f64;
                Ok(Fraction((converted_distance, rounded_row)))
            }
            _ => Err(NoSnellenEquivalent(self.to_string())),
        }
    }
}

impl SnellenEquivalent for ChartRow {
    fn snellen_equivalent(&self) -> VisualAcuityResult<Fraction> {
        match self {
            ChartRow {
                fraction: Some(ref_acuity),
                reference_distance,
                ..
            } => {
                match reference_distance {
                    // Found a chart row + no conversion necessary.
                    DistanceUnits::NotProvided => Ok(ref_acuity.clone()),
                    _ => Err(NoSnellenEquivalent(format!(
                        "Measurement distance required! {self:?}"
                    ))),
                }
            }
            _ => Err(NoSnellenEquivalent(format!("{self:?}"))),
        }
    }
}
