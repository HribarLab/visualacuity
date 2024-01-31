#![allow(non_camel_case_types)]

extern crate visualacuity;

use std::collections::BTreeMap;
use pyo3;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::types::*;

mod types;

#[pymodule]
#[pyo3(name = "_lib")]
fn visualacuity_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Parser>()?;
    m.add_class::<VisitNote>()?;
    m.add_class::<Laterality>()?;
    m.add_class::<DistanceOfMeasurement>()?;
    m.add_class::<Correction>()?;
    m.add_class::<Method>()?;
    m.add_class::<PinHole>()?;
    Ok(())
}

#[pyclass]
pub struct Parser {
    parser: visualacuity::Parser
}

#[pymethods]
impl Parser {
    #[new]
    pub fn new() -> anyhow::Result<Self> {
        let parser = visualacuity::Parser::new();
        Ok(Self { parser })
    }

    pub fn parse_visit(&self, notes: BTreeMap<&str, &str>) -> PyResult<BTreeMap<String, VisitNote>> {
        match self.parser.parse_visit(notes.into()) {
            Ok(result) => Ok(result.into_iter().map(|(key, v)| (key.into(), v.into())).collect()),
            Err(e) => Err(PyValueError::new_err(format!("{:?}", e)))
        }
    }
}
