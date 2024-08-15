use std::collections::BTreeMap;

use crate::errors::OptionResult;
use crate::{Visit, VisitNote};

pub(crate) trait RoundPlaces {
    fn round_places(&self, p: usize) -> Self;
}

impl RoundPlaces for f64 {
    fn round_places(&self, p: usize) -> f64 {
        let scalar = 10.0_f64.powf(p as f64);
        (self * scalar).round() / scalar
    }
}

impl<T: RoundPlaces> RoundPlaces for Option<T> {
    fn round_places(&self, p: usize) -> Self {
        match self {
            Some(x) => Some(x.round_places(p)),
            None => None,
        }
    }
}

impl<T: RoundPlaces + Clone> RoundPlaces for OptionResult<T> {
    fn round_places(&self, p: usize) -> Self {
        match self {
            Self::Some(x) => Self::Some(x.round_places(p)),
            _ => self.clone(),
        }
    }
}

impl<T: RoundPlaces, E: Clone> RoundPlaces for Result<T, E> {
    fn round_places(&self, p: usize) -> Self {
        match self {
            Ok(x) => Ok(x.round_places(p)),
            Err(e) => Err(e.clone()),
        }
    }
}

impl RoundPlaces for VisitNote {
    fn round_places(&self, p: usize) -> Self {
        let mut result = self.clone();
        result.log_mar_base = result.log_mar_base.round_places(p);
        result.log_mar_base_plus_letters = result.log_mar_base_plus_letters.round_places(p);
        result
    }
}

impl<K: Clone + Ord, V: RoundPlaces> RoundPlaces for BTreeMap<K, V> {
    fn round_places(&self, p: usize) -> Self {
        self.iter()
            .map(|(k, v)| (k.clone(), v.round_places(p)))
            .collect()
    }
}

impl RoundPlaces for Visit {
    fn round_places(&self, p: usize) -> Self {
        Self(self.0.round_places(p))
    }
}
