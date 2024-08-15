use lalrpop_util;

lalrpop_util::lalrpop_mod!(
    // synthesized by LALRPOP
    #[allow(unused_imports)]
    #[allow(dead_code)]
    grammar,
    "/parser/grammar.rs"
);

lalrpop_util::lalrpop_mod!(
    // synthesized by LALRPOP
    #[allow(unused_imports)]
    #[allow(dead_code)]
    key,
    "/parser/key.rs"
);

mod decorator;
mod grammar_helpers;
mod wrapper;

pub(crate) use decorator::Content;
pub(crate) use wrapper::*;
