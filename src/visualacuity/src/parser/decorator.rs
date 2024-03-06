use std::fmt::{Display, Formatter};

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub struct Expression {
    pub original: String,
    pub converted: String,
}

impl Expression {
    pub fn new<T: ToString>(original: T) -> Self {
        Self { original: original.to_string(), converted: original.to_string() }
    }

    pub fn rewrite<T: ToString>(self, converted: T) -> Self {
        Self { converted: converted.to_string(), ..self }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.converted)
    }
}
