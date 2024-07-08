pub use chart::ChartRow;

mod chart;


#[cfg(test)]
mod tests {
    use crate::VisualAcuityResult;
    use super::*;
    use chart::Chart;

    #[test]
    fn test_load() -> VisualAcuityResult<()>{
        let bailey_lovie = Chart::load("bailey-lovie")?;
        let actual = bailey_lovie.get_row("20/20").map(|r| r.next_log_mar).flatten();
        assert_eq!(actual, Some(-0.1));
        Ok(())
    }
}
