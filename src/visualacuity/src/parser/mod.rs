use lalrpop_util;


lalrpop_util::lalrpop_mod!(
// synthesized by LALRPOP
#[allow(unused_imports)]
#[allow(dead_code)]
grammar, "/parser/grammar.rs"
);

mod grammar_helpers;
mod decorator;
mod wrapper;

pub(crate) use wrapper::*;
pub(crate) use decorator::Content;
