use crate::charts::chart::ChartRow;
use crate::logmar::LogMarBase;
use crate::snellen_equivalent::SnellenEquivalent;
use crate::structure::Fraction;
use crate::VisualAcuityError::NoSnellenEquivalent;
use crate::VisualAcuityResult;

impl LogMarBase for ChartRow {
    fn log_mar_base(&self) -> VisualAcuityResult<f64> {
        let Some(log_mar) = self.log_mar else { return Err(NoSnellenEquivalent("".to_string())); };
        Ok(log_mar)
    }
}

impl SnellenEquivalent for ChartRow {
    fn snellen_equivalent(&self) -> VisualAcuityResult<Fraction> {
        let Some(fraction) = self.fraction else { return Err(NoSnellenEquivalent("".to_string())) };
        Ok(fraction)
    }
}

#[cfg(test)]
mod tests {
    use crate::charts::chart::Chart;
    use crate::VisualAcuityResult;
    use super::*;

    #[test]
    fn test_snellen_logmar() -> VisualAcuityResult<()>{
        let chart = Chart::load("bailey-lovie")?;
        let va = chart.get_row("20/20").expect("");
        let log_mar = va.log_mar_base();
        assert_eq!(log_mar, Ok(0.0));
        Ok(())
    }

    #[test]
    fn test_snellen_equivalent() -> VisualAcuityResult<()>{
        let chart = Chart::load("bailey-lovie")?;
        let va = chart.get_row("20/20").expect("");
        let log_mar = va.snellen_equivalent();
        assert_eq!(log_mar, Ok((20, 20).into()));
        Ok(())
    }
}