use aho_corasick::{AhoCorasick, Match, PatternID};
use strex_parser::StrexHir;

pub struct StrexSet {
    aho: AhoCorasick,
}

impl StrexSet {
    pub fn new<I, S>(strexes: I) -> Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        let words = strexes
            .into_iter()
            .flat_map(|strex| {
                let hir = StrexHir::parse(strex.as_ref()).unwrap();
                hir.words()
            })
            .collect::<Vec<_>>();
        Self {
            aho: AhoCorasick::new(words).unwrap(),
        }
    }

    pub fn matches(&self, haystack: &str) -> impl Iterator<Item = Match> {
        self.aho.find_iter(haystack)
    }
}
