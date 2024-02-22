use std::collections::BTreeMap;
use std::fmt::Display;
use lazy_static::lazy_static;
use std::str::FromStr;
use itertools::Itertools;
use crate::{DistanceUnits, Fraction};
use crate::VisualAcuityError::{ChartNotFound, LogMarInvalidSnellenRow, MultipleValues, ParseError};
use crate::{VisualAcuityError, VisualAcuityResult};

lazy_static! {
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
static ref ORDERED_CHARTS: Vec<&'static Chart<'static>> = vec![
        Chart::load("snellen").unwrap(),
        Chart::load("bailey-lovie").unwrap(),
        Chart::load("jaeger").unwrap(),
        Chart::load("teller").unwrap(),
        Chart::load("schulze-bonzel").unwrap(),
        Chart::load("etdrs").unwrap(),
];
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Chart<'a> {
    by_row_number: BTreeMap<i8, ChartRow>,
    by_text: BTreeMap<&'a str, ChartRow>,
}

impl<'a> Chart<'a> {
    pub(crate) fn load(name: &'a str) -> VisualAcuityResult<&Self> {
        match PREDEFINED_CHARTS.get(name) {
            Some(chart) => Ok(chart),
            None => {
                eprintln!("{:?}", PREDEFINED_CHARTS.keys().collect_vec());
                // Here we could check whether there's a filename matching the string?
                Err(ChartNotFound(name.to_string()))
            }
        }
    }

    pub(crate) fn get_row<S: ToString>(&self, text: S) -> Option<&ChartRow> {
        self.by_text.get(&*text.to_string())
    }

    pub(crate) fn parse_files(contents: Vec<(&'a str, &'a str)>) -> VisualAcuityResult<Self> {
        let mut chart = Self::default();
        let mut n_letters = BTreeMap::new();
        let mut log_mars = BTreeMap::new();

        // Collect up some parsed `ChartRow`s
        let rows: Vec<(i8, &str, ChartRow)> = contents.into_iter()
            .flat_map(|(chart_name, content)| {
                map_tsv(content).into_iter()
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
                chart_row.next_n_letters = n_letters.get(&(row_number - 1)).cloned().unwrap_or_default();
                chart_row.next_log_mar = log_mars.get(&(row_number - 1)).cloned().flatten().unwrap_or_default();
                chart_row.prev_log_mar = log_mars.get(&(row_number + 1)).cloned().flatten().unwrap_or_default();

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
    pub(crate) next_n_letters: Option<u8>
}

pub(crate) trait Stringish: ToString + Display + Ord {}
impl Stringish for &str {}
impl Stringish for String {}
impl Stringish for &String {}

impl ChartRow {
    pub(crate) fn find<'a, S: ToString>(value: &S) -> Option<&'a Self> {
        ORDERED_CHARTS.iter()
            .filter_map(|chart| chart.get_row(value.to_string().as_str().trim()))
            .next()
    }

    pub(crate) fn log_mar_increment(&self, plus_letters: i32) -> VisualAcuityResult<f64> {
        if plus_letters == 0 {
            Ok(0.0)
        }
        else if plus_letters > 0 {
            let e = || Err(LogMarInvalidSnellenRow(format!("Missing next row values: {self:?}")));
            let Some(log_mar) = self.log_mar else { return e(); };
            let Some(next_log_mar) = self.next_log_mar else { return e(); };
            let Some(next_n_letters) = self.next_n_letters else { return e(); };
            Ok((next_log_mar - log_mar) / next_n_letters as f64)
        }
        else {
            let e = || Err(LogMarInvalidSnellenRow(format!("Missing previous row values: {self:?}")));
            let Some(log_mar) = self.log_mar else { return e(); };
            let Some(prev_log_mar) = self.prev_log_mar else { return e(); };
            let Some(n_letters) = self.n_letters else { return e(); };
            Ok((log_mar - prev_log_mar) / n_letters as f64)
        }
    }
}

fn parse_row<'a>(chart_name: &'a str, row: BTreeMap<&'a str, &'a str>) -> VisualAcuityResult<(i8, &'a str, ChartRow)> {
    let row_number = row.get("Row").expect("Must contain a row number!").parse()?;
    let row_text = row.get("Text").expect("Must contain text!");
    let fraction = parse_some(nonempty(row.get("Fraction").cloned()))?;
    let log_mar = row.get("LogMAR").map(|&s| s.trim_start_matches('+'));
    let log_mar = parse_some(nonempty(log_mar))?;
    let (fraction, log_mar) = fill_in_log_mar(fraction, log_mar)?;
    let n_letters = parse_some(nonempty(row.get("Letters").cloned()))?;
    let reference_distance = parse_some(nonempty(row.get("Distance").cloned()))?
        .unwrap_or_default();
    let prev_log_mar = None;
    let next_log_mar = None;
    let next_n_letters = None;
    let chart_name = chart_name.to_string();
    let chart_row = ChartRow {
        chart_name, row_number, fraction, reference_distance,
        log_mar, n_letters, prev_log_mar, next_log_mar, next_n_letters
    };
    Ok((row_number, *row_text, chart_row))
}


fn parse_some<T: FromStr>(s: Option<&str>) -> VisualAcuityResult<Option<T>>
    where VisualAcuityError: From<<T as FromStr>::Err>
{
    let Some(s) = s else { return Ok(None); };
    s.parse::<T>().map(|obj| Some(obj)).map_err(|_| ParseError(s.to_string()))
}

fn nonempty(s: Option<&str>) -> Option<&str> {
    match s.map(str::trim) { None => None, Some("") => None, Some(s) => Some(s) }
}

fn fill_in_log_mar(fraction: Option<Fraction>, log_mar: Option<f64>) -> VisualAcuityResult<(Option<Fraction>, Option<f64>)> {
    match (fraction, log_mar) {
        (Some(Fraction((n, d))), None) => Ok((Some(Fraction((n, d))), Some(-(n / d).log10()))),
        _ => Ok((fraction, log_mar))
    }

}


fn load_predefined<'a>(name: &'a str, filenames: Vec<&'a str>) -> VisualAcuityResult<(&'a str, Chart<'a>)> {
    let contents = filenames.into_iter().map(|f| (name, f)).collect();
    let result = Chart::parse_files(contents);
    Ok((name, result?))
}

pub fn map_tsv(contents: &str) -> Vec<(&str, BTreeMap<&str, &str>)> {
    let mut lines = contents.lines().into_iter();
    let header = lines.next().expect("File can't have no lines!").split("\t");
    lines
        .filter_map(|line| match line.trim() { "" => None, l => Some(l) })
        .map(|line| (line, line.split("\t")))
        .map(|(line, columns)| (line, header.clone().zip(columns).collect()))
        .collect()
}


