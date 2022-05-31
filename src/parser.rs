use std::ops::Range;

use chumsky::{prelude::*, text::newline};

#[derive(Debug)]
pub struct Rule {
    pub divisor: u32,
    pub literal: String,
}

#[derive(Debug)]
pub struct Rules {
    pub bounds: Range<i32>,
    pub rules: Vec<Rule>,
}

pub fn parser() -> impl Parser<char, Rules, Error = Simple<char>> {
    let rule = text::int(10)
        .padded()
        .map(|s: String| s.parse().unwrap())
        .then_ignore(just(':').padded())
        .then(take_until(newline()))
        .map(|(i, (c, _))| Rule {
            divisor: i,
            literal: c.iter().cloned().collect::<String>(),
        });

    let colon_int = just(':')
        .padded()
        .ignore_then(text::int(10))
        .map(|s: String| s.parse().unwrap())
        .then_ignore(newline().repeated());

    let bounds_start = just("start")
        .padded()
        .ignore_then(colon_int)
        .labelled("start");
    let bounds_end = just("end").padded().ignore_then(colon_int).labelled("end");

    let bounds = bounds_start.then(bounds_end).map(|(s, e)| s..e);
    let rules = rule.repeated().padded();

    bounds.then(rules).then_ignore(end()).map(|(b, r)| Rules {
        bounds: b,
        rules: r,
    })
}
