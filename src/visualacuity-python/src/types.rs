use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use pyo3::{pyclass, pymethods, PyResult};
use pyo3::exceptions::PyValueError;
use visualacuity::VisualAcuityResult;

#[pyclass(module="visualacuity._lib")]
pub struct VisitNote {
    #[pyo3(get)]
    pub text: String,

    #[pyo3(get)]
    pub text_plus: String,

    #[pyo3(get)]
    pub laterality: Laterality,

    #[pyo3(get)]
    pub distance_of_measurement: DistanceOfMeasurement,

    #[pyo3(get)]
    pub correction: Correction,

    #[pyo3(get)]
    pub method: Method,

    #[pyo3(get)]
    pub plus_letters: Vec<i32>,

    #[pyo3(get)]
    pub extracted_value: String,

    #[pyo3(get)]
    pub pinhole: PinHole,

    snellen_equivalent: VisualAcuityResult<Option<(f64, f64)>>,
    log_mar_base: VisualAcuityResult<Option<f64>>,
    log_mar_base_plus_letters: VisualAcuityResult<Option<f64>>,
}

#[pymethods]
impl VisitNote {
    #[getter]
    fn snellen_equivalent(&self) -> PyResult<Option<(f64, f64)>> {
        self.snellen_equivalent.clone()
            .or_else(|e| Err(PyValueError::new_err(format!("{:?}", e))))
    }

    #[getter]
    fn log_mar_base(&self) -> PyResult<Option<f64>> {
        self.log_mar_base.clone()
            .or_else(|e| Err(PyValueError::new_err(format!("{:?}", e))))
    }

    #[getter]
    fn log_mar_base_plus_letters(&self) -> PyResult<Option<f64>> {
        self.log_mar_base_plus_letters.clone()
            .or_else(|e| Err(PyValueError::new_err(format!("{:?}", e))))
    }
}

impl From<visualacuity::VisitNote> for VisitNote {
    fn from(value: visualacuity::VisitNote) -> Self {
        Self {
            text: value.text.to_string(),
            text_plus: value.text_plus.to_string(),
            laterality: value.laterality.into(),
            distance_of_measurement: value.distance_of_measurement.into(),
            correction: value.correction.into(),
            pinhole: value.pinhole.into(),
            method: value.method.into(),
            extracted_value: value.extracted_value,
            snellen_equivalent: value.snellen_equivalent.map(|se| se.map(Into::into)),
            log_mar_base: value.log_mar_base,
            log_mar_base_plus_letters: value.log_mar_base_plus_letters,
            plus_letters: value.plus_letters.into()
        }
    }
}

#[pyclass(module="visualacuity")]
#[derive(Hash, Clone, PartialEq, Debug)]
pub enum Laterality {
    ERROR = 0,
    UNKNOWN = 1,
    OS = 2,
    OD = 3,
    OU = 4,
}

#[pymethods]
impl Laterality {
    fn __hash__(&self) -> u64 {
        py_hash(self)
    }
}

impl From<visualacuity::Laterality> for Laterality {
    fn from(value: visualacuity::Laterality) -> Self {
        use visualacuity::Laterality::*;
        match value {
            Error(_) => Self::ERROR,
            Unknown => Self::UNKNOWN,
            OS => Self::OS,
            OD => Self::OD,
            OU => Self::OU,
        }
    }
}

#[pyclass(module="visualacuity")]
#[derive(Hash, Clone, PartialEq, Debug)]
pub enum DistanceOfMeasurement {
    ERROR = 0,
    UNKNOWN = 1,
    NEAR = 2,
    DISTANCE = 3,
}

#[pymethods]
impl DistanceOfMeasurement {
    fn __hash__(&self) -> u64 {
        py_hash(self)
    }
}

impl From<visualacuity::DistanceOfMeasurement> for DistanceOfMeasurement {
    fn from(value: visualacuity::DistanceOfMeasurement) -> Self {
        use visualacuity::DistanceOfMeasurement::*;
        match value {
            Error(_) => Self::ERROR,
            Unknown => Self::UNKNOWN,
            Near => Self::NEAR,
            Distance => Self::DISTANCE,
        }
    }
}

#[pyclass(module="visualacuity")]
#[derive(Hash, Clone, PartialEq, Debug)]
pub enum Correction {
    ERROR = 0,
    UNKNOWN = 1,
    CC = 2,
    SC = 3,
}

#[pymethods]
impl Correction {
    fn __hash__(&self) -> u64 {
        py_hash(self)
    }
}

impl From<visualacuity::Correction> for Correction {
    fn from(value: visualacuity::Correction) -> Self {
        match value {
            visualacuity::Correction::Error(_) => Self::ERROR,
            visualacuity::Correction::Unknown => Self::UNKNOWN,
            visualacuity::Correction::CC => Self::CC,
            visualacuity::Correction::SC => Self::SC,
        }
    }
}

#[pyclass(module="visualacuity")]
#[derive(Hash, Clone, PartialEq, Debug)]
pub enum Method {
    ERROR = 0,
    UNKNOWN = 1,
    SNELLEN = 2,
    JAEGER = 3,
    ETDRS = 4,
    TELLER = 5,
    LOW_VISION = 6,
    PIN_HOLE = 7,
    BINOCULAR = 8,
    NOT_TAKEN = 9,
}

#[pymethods]
impl Method {
    fn __hash__(&self) -> u64 {
        py_hash(self)
    }
}

impl From<visualacuity::Method> for Method {
    fn from(value: visualacuity::Method) -> Self {
        use visualacuity::Method::*;
        match value {
            Error(_) => Self::ERROR,
            Unknown => Self::UNKNOWN,
            Snellen => Self::SNELLEN,
            Jaeger => Self::JAEGER,
            ETDRS => Self::ETDRS,
            Teller => Self::TELLER,
            LowVision => Self::LOW_VISION,
            PinHole => Self::PIN_HOLE,
            Binocular => Self::BINOCULAR,
            NotTaken => Self::NOT_TAKEN,
        }
    }
}

#[pyclass(module="visualacuity")]
#[derive(Hash, Clone, PartialEq, Debug)]
pub enum PinHole {
    ERROR = 0,
    UNKNOWN = 1,
    WITH = 2,
    WITHOUT = 3,
}

#[pymethods]
impl PinHole {
    fn __hash__(&self) -> u64 {
        py_hash(self)
    }
}


impl From<visualacuity::PinHole> for PinHole {
    fn from(value: visualacuity::PinHole) -> Self {
        use visualacuity::PinHole::*;
        match value {
            Error(_) => Self::ERROR,
            Unknown => Self::UNKNOWN,
            With => Self::WITH,
            Without => Self::WITHOUT,
        }
    }
}

fn py_hash<T: Hash>(obj: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}
