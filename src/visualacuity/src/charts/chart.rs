use crate::logmar::LogMarBase;
use crate::VisualAcuityError::{ChartNotFound, MultipleValues, ParseError};
use crate::{DistanceUnits, Fraction};
use crate::{VisualAcuityError, VisualAcuityResult};
use itertools::Itertools;
use lazy_static::lazy_static;
use std::collections::BTreeMap;
use std::str::FromStr;

lazy_static! {
    // Pre-load data from the chart definition files in ../../assets/charts
    static ref PREDEFINED_CHARTS: BTreeMap<&'static str, Chart<'static>> = [
            load_predefined("snellen", vec![
                include_str!("../../assets/charts/snellen.feet.tsv"),
            ]),
            load_predefined("bailey-lovie", vec![
                include_str!("../../assets/charts/bailey-lovie.feet.tsv"),
                include_str!("../../assets/charts/bailey-lovie.meters.tsv"),
            ]),
            load_predefined("jaeger", vec![
                include_str!("../../assets/charts/jaeger.tsv"),
            ]),
            load_predefined("teller", vec![
                include_str!("../../assets/charts/teller.feet.tsv"),
                include_str!("../../assets/charts/teller.cycm.tsv"),
                include_str!("../../assets/charts/teller.card.tsv"),
            ]),
            load_predefined("schulze-bonzel", vec![
                include_str!("../../assets/charts/schulze-bonzel.tnlv.tsv"),
            ]),
            load_predefined("etdrs", vec![
                include_str!("../../assets/charts/etdrs.tsv"),
            ]),
        ].into_iter().try_collect().unwrap();

    // Establish an ordered lookup for the above charts. If a value occurs in multiple charts,
    // prefer the first. This is hard-coded for now, but will probably become configurable
    // by the API consumer as we come to understand what people need.
    static ref ORDERED_CHARTS: Vec<&'static Chart<'static>> = vec![
            Chart::load("snellen").unwrap(),
            Chart::load("bailey-lovie").unwrap(),
            Chart::load("jaeger").unwrap(),
            Chart::load("teller").unwrap(),
            Chart::load("schulze-bonzel").unwrap(),
            Chart::load("etdrs").unwrap(),
    ];
}

/// A `Chart` here is basically a ``data dictionary'' in which all the entries are related.
/// Each entry is expected to have a `row_number`, which can be used to build logical
/// relationships between the entries (e.g. for input text like "20/10 -2", computing LogMAR for
/// partially-read lines requires information about two rows: 20/10 and 20/12.5)
#[derive(Clone, PartialEq, Default, Debug)]
pub struct Chart<'a> {
    by_row_number: BTreeMap<i8, ChartRow>,
    by_text: BTreeMap<&'a str, ChartRow>,
}

impl<'a> Chart<'a> {
    /// Retrieve a `ChartRow` by its normalized "data dictionary" text
    pub(crate) fn get_row<S: ToString>(&self, text: S) -> Option<&ChartRow> {
        self.by_text.get(&*text.to_string())
    }

    /// Load a `Chart` from the contents of a TSV file
    pub(crate) fn load(name: &'a str) -> VisualAcuityResult<&Self> {
        match PREDEFINED_CHARTS.get(name) {
            Some(chart) => Ok(chart),
            None => {
                // Here we could check whether there's a filename matching the string?
                Err(ChartNotFound(name.to_string()))
            }
        }
    }

    pub(crate) fn parse_files(contents: Vec<(&'a str, &'a str)>) -> VisualAcuityResult<Self> {
        let mut chart = Self::default();
        let mut n_letters = BTreeMap::new();
        let mut log_mars = BTreeMap::new();

        // Collect up some parsed `ChartRow`s
        let rows: Vec<(i8, &str, ChartRow)> = contents
            .into_iter()
            .flat_map(|(chart_name, content)| {
                map_tsv(content)
                    .into_iter()
                    .map(|r| (chart_name, r))
                    .collect_vec()
            })
            .map(|(chart_name, (line, tsv_row))| {
                parse_row(chart_name, tsv_row).map_err(|_| ParseError(line.to_string()))
            })
            .map_ok(|(row_number, row_text, chart_row)| {
                n_letters.insert(row_number, chart_row.n_letters.clone());
                log_mars.insert(row_number, Some(chart_row.log_mar.clone()));
                (row_number, row_text, chart_row)
            })
            .try_collect()?;

        for (row_number, row_text, mut chart_row) in rows {
            chart_row.next_n_letters = n_letters
                .get(&(row_number - 1))
                .cloned()
                .unwrap_or_default();
            chart_row.next_log_mar = log_mars
                .get(&(row_number - 1))
                .cloned()
                .flatten()
                .unwrap_or_default();
            chart_row.prev_log_mar = log_mars
                .get(&(row_number + 1))
                .cloned()
                .flatten()
                .unwrap_or_default();

            let found = chart.by_row_number.insert(row_number, chart_row.clone());
            if found.is_some() && found.as_ref() != Some(&chart_row) {
                return Err(MultipleValues(format!("{chart_row:?}")));
            }
            chart.by_text.insert(row_text, chart_row);
        }
        // }
        Ok(chart)
    }
}

/// A `ChartRow` is a single entry in a `Chart`, and one line from a TSV file. Additional
/// information about adjacent lines is precomputed here for easy access, namely `prev_log_mar`,
/// `next_log_mar`, and `next_n_letters`.
#[derive(Clone, PartialEq, Debug)]
pub struct ChartRow {
    pub(crate) chart_name: String,
    pub(crate) row_number: i8,
    pub(crate) fraction: Option<Fraction>,
    pub(crate) reference_distance: DistanceUnits,
    pub(crate) log_mar: Option<f64>,
    pub(crate) n_letters: Option<u8>,
    pub(crate) prev_log_mar: Option<f64>,
    pub(crate) next_log_mar: Option<f64>,
    pub(crate) next_n_letters: Option<u8>,
}

impl ChartRow {
    /// Static lookup for a chart row by normalized text, searching all pre-loaded `Chart`
    /// definitions. Chart priority is specified in `ORDERED_CHARTS` above.
    pub(crate) fn find<'a, S: ToString>(value: &S) -> Option<&'a Self> {
        ORDERED_CHARTS
            .iter()
            .filter_map(|chart| chart.get_row(value.to_string().as_str().trim()))
            .next()
    }
}

fn parse_row<'a>(
    chart_name: &'a str,
    row: BTreeMap<&'a str, &'a str>,
) -> VisualAcuityResult<(i8, &'a str, ChartRow)> {
    let row_number = row
        .get("Row")
        .expect("Must contain a row number!")
        .parse()?;
    let row_text = row.get("Text").expect("Must contain text!");
    let fraction = parse_some(nonempty(row.get("Fraction").cloned()))?;
    let log_mar = row.get("LogMAR").map(|&s| s.trim_start_matches('+'));
    let log_mar = parse_some(nonempty(log_mar))?;
    let (fraction, log_mar) = fill_in_log_mar(fraction, log_mar)?;
    let n_letters = parse_some(nonempty(row.get("Letters").cloned()))?;
    let reference_distance =
        parse_some(nonempty(row.get("Distance").cloned()))?.unwrap_or_default();
    let prev_log_mar = None;
    let next_log_mar = None;
    let next_n_letters = None;
    let chart_name = chart_name.to_string();
    let chart_row = ChartRow {
        chart_name,
        row_number,
        fraction,
        reference_distance,
        log_mar,
        n_letters,
        prev_log_mar,
        next_log_mar,
        next_n_letters,
    };
    Ok((row_number, *row_text, chart_row))
}

fn parse_some<T: FromStr>(s: Option<&str>) -> VisualAcuityResult<Option<T>>
where
    VisualAcuityError: From<<T as FromStr>::Err>,
{
    let Some(s) = s else {
        return Ok(None);
    };
    s.parse::<T>()
        .map(|obj| Some(obj))
        .map_err(|_| ParseError(s.to_string()))
}

fn nonempty(s: Option<&str>) -> Option<&str> {
    match s.map(str::trim) {
        None => None,
        Some("") => None,
        Some(s) => Some(s),
    }
}

fn fill_in_log_mar(
    fraction: Option<Fraction>,
    log_mar: Option<f64>,
) -> VisualAcuityResult<(Option<Fraction>, Option<f64>)> {
    match (fraction, log_mar) {
        (Some(fraction), None) => Ok((Some(fraction), Some(fraction.log_mar_base()?))),
        _ => Ok((fraction, log_mar)),
    }
}

fn load_predefined<'a>(
    name: &'a str,
    filenames: Vec<&'a str>,
) -> VisualAcuityResult<(&'a str, Chart<'a>)> {
    let contents = filenames.into_iter().map(|f| (name, f)).collect();
    let result = Chart::parse_files(contents);
    Ok((name, result?))
}

pub fn map_tsv(contents: &str) -> Vec<(&str, BTreeMap<&str, &str>)> {
    let mut lines = contents.lines().into_iter();
    let header = lines.next().expect("File can't have no lines!").split("\t");
    lines
        .filter_map(|line| match line.trim() {
            "" => None,
            l => Some(l),
        })
        .map(|line| (line, line.split("\t")))
        .map(|(line, columns)| (line, header.clone().zip(columns).collect()))
        .collect()
}
