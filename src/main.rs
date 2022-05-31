use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{error as ChumskyError, Parser};

use fizzy::parser;

fn main() {
    let src = r#"
        start: 1
        end: 100

        3: fizz
        5: buzz
    "#;
    match parser().parse(src) {
        Ok(rules) => {
            println!("{:?}", rules);
        }
        Err(errs) => errs.into_iter().for_each(|e| {
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
                                "Unexpected token {}",
                                e.found()
                                    .map(char::to_string)
                                    .unwrap_or_else(|| "end of file".to_string())
                                    .fg(Color::Red)
                            ))
                            .with_color(Color::Red),
                    ),
                ChumskyError::SimpleReason::Custom(msg) => reporter.with_message(msg).with_label(
                    Label::new(e.span())
                        .with_message(format!("{}", msg.fg(Color::Red)))
                        .with_color(Color::Red),
                ),
                ChumskyError::SimpleReason::Unclosed { .. } => unreachable!(),
            };

            report.finish().print(Source::from(src)).unwrap();
        }),
    }
}
