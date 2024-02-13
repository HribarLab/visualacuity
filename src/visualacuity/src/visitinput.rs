use std::collections::{BTreeMap, HashMap};
use std::collections::btree_map::IntoIter;
use regex::Regex;
use itertools::Itertools;
use crate::cache::LruCacher;

#[derive(Clone, PartialEq, Debug)]
pub struct VisitInput(BTreeMap<String, String>);

impl VisitInput {
    pub fn get_str(&self, key: &str) -> Option<&str> { self.0.get(key).map(|s| s.as_str()) }
}

impl<A: ToString, B: ToString, I: IntoIterator<Item=(A, B)>> From<I> for VisitInput {
    fn from(iter: I) -> Self {
        Self(iter.into_iter().map(|(a, b)| (a.to_string(), b.to_string())).collect())
    }
}

#[derive(PartialEq, Debug)]
pub(crate) struct VisitInputMerged(pub(crate) BTreeMap<String, (String, String)>);

impl VisitInputMerged {
    pub(crate) fn into_iter(self) -> IntoIter<String, (String, String)> { self.0.into_iter() }
}

impl<A: ToString, B: ToString, C: ToString, I: IntoIterator<Item=(A, (B, C))>> From<I> for VisitInputMerged {
    fn from(iter: I) -> Self {
        let map = iter
            .into_iter()
            .map(|(a, (b, c))| (a.to_string(), (b.to_string(), c.to_string())))
            .collect();
        Self(map)
    }
}

pub(crate) struct ColumnMerger {
    pattern: Regex,
    mapping_cache: LruCacher<String, HashMap<String, Option<String>>>
}

impl ColumnMerger {
    pub(crate) fn new(cache_size: usize) -> Self {
        let pattern = Regex::new(r"(?i)^(.*?)\s*(\+/-|\+|\splus)$").expect("");
        let mapping_cache = LruCacher::new(cache_size);
        Self { pattern, mapping_cache }
    }

    pub(crate) fn merge_plus_columns<'a>(&'a self, notes: VisitInput) -> VisitInputMerged {
        self.key_mapping(&notes)
            .into_iter()
            .map(|(text_key, text_plus_key)| {
                let default = "";
                let text = notes.get_str(&text_key);
                let text_plus = text_plus_key.and_then(|k| notes.get_str(&k));
                (text_key, (text.unwrap_or(default), text_plus.unwrap_or(default)))
            })
            .into()
    }

    pub(crate) fn key_mapping(&self, collection: &VisitInput) -> HashMap<String, Option<String>> {
        let cache_key = format!("{:?}", collection.0.keys().collect_vec());
        self.mapping_cache.get(&cache_key, || {
            collection.0.iter()
                .map(|(key, _)| {
                    let key= key.clone();
                    let parent_key = self.pattern.captures(&key)
                        .and_then(|c| c.get(1))
                        .map(|m| m.as_str().to_string());

                    match parent_key {
                        None => (key, None),
                        Some(pk) => (pk, Some(key))
                    }
                })
                .sorted()
                .collect()
        })
    }
}
