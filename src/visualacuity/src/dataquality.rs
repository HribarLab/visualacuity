#[derive(PartialEq, Debug, Clone, Default)]
pub enum DataQuality {
    Exact,
    Convertible,
    #[default]
    Unrecognized,
}

impl DataQuality {
    pub(crate) fn from_vec(items: Vec<DataQuality>) -> Self
    {
        items.first().cloned().unwrap_or_default()
    }
}