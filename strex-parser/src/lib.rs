use chumsky::prelude::*;
use itertools::Itertools;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrexHir {
    Literal(String),
    Or(Vec<StrexHir>),
    Concat(Vec<StrexHir>),
    Wild,
}

impl Display for StrexHir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrexHir::Literal(lit) => f.write_str(lit),
            StrexHir::Or(strex_hirs) => {
                write!(f, "({})", strex_hirs.iter().join("|"))
            }
            StrexHir::Concat(strex_hirs) => {
                write!(f, "{}", strex_hirs.iter().join(""))
            }
            StrexHir::Wild => f.write_str(".*"),
        }
    }
}

fn parser<'a>() -> impl Parser<'a, &'a str, StrexHir, extra::Err<Simple<'a, char>>> {
    recursive(|bf| {
        choice((
            just(".*").to(StrexHir::Wild),
            text::ident().map(|string: &str| StrexHir::Literal(string.to_owned())),
            bf.delimited_by(just('('), just(')')),
        ))
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|mut items| {
            if items.len() == 1 {
                items.pop().unwrap()
            } else {
                StrexHir::Concat(items)
            }
        })
        .separated_by(just("|"))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|mut items| {
            if items.len() == 1 {
                items.pop().unwrap()
            } else {
                StrexHir::Or(items)
            }
        })
    })
}

impl StrexHir {
    pub fn parse(strex: &str) -> Option<Self> {
        parser().parse(strex).output().cloned()
    }

    pub fn words(&self) -> Vec<String> {
        match self {
            StrexHir::Literal(l) => vec![l.clone()],
            StrexHir::Concat(strex_hirs) | StrexHir::Or(strex_hirs) => {
                strex_hirs.iter().flat_map(StrexHir::words).collect()
            }
            StrexHir::Wild => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::StrexHir;

    #[track_caller]
    fn assert_parse_pretty(parse: &str, pretty: &str) {
        assert_eq!(StrexHir::parse(parse).unwrap().to_string(), pretty);
    }

    #[track_caller]
    fn assert_pretty(pretty: &str) {
        assert_parse_pretty(pretty, pretty);
    }

    #[test]
    fn parse_and_pretty_print_equal() {
        assert_pretty(".*");
        assert_pretty("foo.*bar.*baz");
        assert_pretty("(foo|faaa).*bar.*baz");
    }
}
