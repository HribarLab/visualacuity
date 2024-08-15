#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Default)]
pub enum DataQuality {
    #[default]
    NoValue = 0,
    CrossReference = 1,
    Exact = 2,
    Multiple = 3,
    ConvertibleConfident = 4,
    ConvertibleFuzzy = 5,
}
