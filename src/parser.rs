use std::process::exit;

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{error as ChumskyError, Parser};
use chumsky::{prelude::*, text::newline};

#[derive(Debug)]
pub struct Rule {
    pub divisor: u32,
    pub literal: String,
}

#[derive(Debug)]
pub struct Bound {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug)]
pub struct Rules {
    pub bounds: Bound,
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

    let bounds = bounds_start
        .then(bounds_end)
        .map(|(s, e)| Bound { start: s, end: e });
    let rules = rule.repeated().padded();

    bounds.then(rules).then_ignore(end()).map(|(b, r)| Rules {
        bounds: b,
        rules: r,
    })
}

pub fn parse(src: &str) -> Rules {
    match parser().parse(src) {
        Ok(rules) => rules,
        Err(errs) => {
            errs.into_iter().for_each(|e| {
                let reporter = Report::build(ReportKind::Error, (), e.span().start);
                let report = match e.reason() {
                    ChumskyError::SimpleReason::Unexpected => reporter
                        .with_message(format!(
                            "{}, expected '{}'",
                            if e.found().is_some() {
                                "Unexpected token in input"
                            } else {
                                "Unexpected end of input"
                            },
                            if let Some(l) = e.label() {
                                l.to_owned()
                            } else if e.expected().len() == 0 {
                                "something else".to_string()
                            } else {
                                e.expected()
                                    .map(|expected| match expected {
                                        Some(expected) => expected.to_string(),
                                        None => "end of input".to_string(),
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            }
                        ))
                        .with_label(
                            Label::new(e.span())
                                .with_message(format!(
                                    "Unexpected token '{}'",
                                    e.found()
                                        .map(char::to_string)
                                        .unwrap_or_else(|| "end of file".to_string())
                                        .fg(Color::Red)
                                ))
                                .with_color(Color::Red),
                        ),
                    ChumskyError::SimpleReason::Custom(msg) => {
                        reporter.with_message(msg).with_label(
                            Label::new(e.span())
                                .with_message(format!("{}", msg.fg(Color::Red)))
                                .with_color(Color::Red),
                        )
                    }
                    ChumskyError::SimpleReason::Unclosed { .. } => unreachable!(),
                };

                report.finish().print(Source::from(src)).unwrap();
            });
            exit(-1);
        }
    }
}
