use std::{collections::HashSet, time::Instant};

use clap::Parser;
use itertools::Itertools;
use rand::random_range;
use regex::RegexSet;

#[derive(Parser)]
enum Command {
    Generate { count: usize },
    RustRegex,
}

fn random_regex_linear() -> String {
    (0..random_range(6..9))
        .map(|_| generate(random_range(16..20)))
        .join(".*")
}

fn random_regex_linear_conditional() -> String {
    (0..random_range(6..9))
        .map(|_| {
            format!(
                "({})",
                (0..random_range(2..4))
                    .map(|_| generate(random_range(17..21)))
                    .join("|")
            )
        })
        .join(".*")
}

fn duration(job: impl FnOnce()) {
    let instant = Instant::now();
    job();
    dbg!(instant.elapsed());
}

fn main() {
    let cmd = Command::parse();

    match cmd {
        Command::Generate { count } => {
            let s = generate(count);
            println!("{s}");
        }
        Command::RustRegex => {
            let regexes = (0..100)
                .map(|_| random_regex_linear())
                .chain((0..100).map(|_| random_regex_linear_conditional()))
                .collect_vec();
            let haystack = generate(1_000_000);
            let regex_set = RegexSet::new(&regexes).unwrap();
            // duration(|| {
            //     let matches = regex_set.matches(&haystack);
            //     dbg!(matches.iter().count());
            // });

            use hyperscan::prelude::*;

            let patterns = Patterns(
                regexes
                    .iter()
                    .map(|regex| {
                        pattern! {regex; CASELESS | SOM_LEFTMOST}
                    })
                    .collect(),
            );
            let db: BlockDatabase = patterns.build().unwrap();
            let scratch = db.alloc_scratch().unwrap();

            duration(|| {
                let mut cnt = HashSet::new();

                db.scan(&haystack, &scratch, |id, _, _, _| {
                    cnt.insert(id);
                    Matching::Continue
                })
                .unwrap();

                dbg!(cnt.len());
            });
        }
    }
}

fn generate(count: usize) -> String {
    (0..count)
        .map(|_| if rand::random_bool(0.5) { 'b' } else { 'a' })
        .collect()
}
