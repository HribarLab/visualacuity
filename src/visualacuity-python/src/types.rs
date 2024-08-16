use std::collections::BTreeMap;
use std::fmt::Debug;

use pyo3::prelude::*;
use pyo3::types::PyTuple;

pub(crate) struct PyWrap<T>(pub(crate) T);

impl<T> From<T> for PyWrap<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: IntoPy<PyObject>, E: Debug> IntoPy<PyObject> for PyWrap<Result<T, E>> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self.0 {
            Ok(obj) => obj.into_py(py),
            Err(_) => "Error".into_py(py),
        }
    }
}

impl<T: IntoPy<PyObject>> IntoPy<PyObject> for PyWrap<visualacuity::OptionResult<T>> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self.0 {
            visualacuity::OptionResult::Some(obj) => obj.into_py(py),
            visualacuity::OptionResult::None => None::<()>.into_py(py),
            visualacuity::OptionResult::Err(_) => "Error".into_py(py),
        }
    }
}

macro_rules! pywrap_enum {
    ($name:ident) => {
        impl IntoPy<PyObject> for PyWrap<visualacuity::$name> {
            fn into_py(self, py: Python<'_>) -> PyObject {
                py_call(py, stringify!($name), (format!("{:?}", self.0),)).expect("boilerplate")
            }
        }
    };
}

pywrap_enum!(DataQuality);
pywrap_enum!(Laterality);
pywrap_enum!(DistanceOfMeasurement);
pywrap_enum!(Correction);
pywrap_enum!(PinHole);
pywrap_enum!(VAFormat);

impl IntoPy<PyObject> for PyWrap<visualacuity::Visit> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let dict: BTreeMap<_, _> = self.0
            .into_iter()
            .map(|(key, entry)| (key, entry.map(PyWrap)))
            .collect();
        py_call(py, "Visit", (dict,)).expect("boilerplate")
    }
}

impl IntoPy<PyObject> for PyWrap<visualacuity::Fraction> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let (distance, row) = *(self.0);
        py_call(py, "SnellenFraction", (distance, row)).expect("boilerplate")
    }
}

impl IntoPy<PyObject> for PyWrap<visualacuity::VisitNote> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        py_call(
            py,
            "VisitNote",
            PyTuple::new(
                py,
                vec![
                    self.0.text.into_py(py),
                    self.0.text_plus.into_py(py),
                    PyWrap(self.0.data_quality).into_py(py),
                    PyWrap(self.0.laterality).into_py(py),
                    PyWrap(self.0.distance_of_measurement).into_py(py),
                    PyWrap(self.0.correction).into_py(py),
                    PyWrap(self.0.pinhole).into_py(py),
                    PyWrap(self.0.va_format.map(PyWrap)).into_py(py),
                    self.0.plus_letters.into_py(py),
                    self.0.extracted_value.into_py(py),
                    PyWrap(self.0.snellen_equivalent.map(PyWrap)).into_py(py),
                    PyWrap(self.0.log_mar_base).into_py(py),
                    PyWrap(self.0.log_mar_base_plus_letters).into_py(py),
                ],
            ),
        )
        .expect("boilerplate")
    }
}
fn py_call(py: Python<'_>, cls: &str, args: impl IntoPy<Py<PyTuple>>) -> PyResult<PyObject> {
    let module = PyModule::import(py, "visualacuity")?;
    let t = module.getattr(cls)?;
    let result = t.call1(args)?;
    Ok(result.into())
}
