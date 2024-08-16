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
    Ok(())
}

#[pyclass]
struct Parser {
    parser: visualacuity::Parser
}

#[pymethods]
impl Parser {
    #[new]
    fn new() -> PyResult<Self> {
        let parser = visualacuity::Parser::new();
        Ok(Self { parser })
    }

    fn parse_visit(
        &self,
        notes: BTreeMap<String, String>,
    ) -> PyResult<PyWrap<visualacuity::Visit>> {
        match self.parser.parse_visit(notes.into()) {
            Ok(result) => Ok(result.into()),
            Err(e) => Err(PyValueError::new_err(format!("{:?}", e))),
        }
    }
}
