use std::{collections::HashSet, ops::Range, time::Instant};

use clap::Parser;
use itertools::Itertools;
use rand::random_range;
use regex::RegexSet;
use strex::StrexSet;

#[derive(Parser)]
enum Command {
    Generate {
        count: usize,
    },
    RustRegex {
        #[arg(long, default_value = "100")]
        linear_regexes: usize,
        #[arg(long, default_value = "100")]
        linear_conditional_regexes: usize,
        #[arg(long, default_value = "1000000")]
        haystack_size: usize,
        #[arg(long, default_value = "6..9", value_parser = parse_range)]
        linear_part_count: Range<usize>,
        #[arg(long, default_value = "16..20", value_parser = parse_range)]
        linear_part_size: Range<usize>,
    },
}

fn parse_range(s: &str) -> Result<Range<usize>, String> {
    let (start, end) = s
        .split_once("..")
        .ok_or("range must be in the form start..end")?;

    let start: usize = start.parse().map_err(|_| "invalid start of range")?;

    let end: usize = end.parse().map_err(|_| "invalid end of range")?;

    if start >= end {
        return Err("range start must be < end".into());
    }

    Ok(start..end)
}

fn random_regex_linear(linear_part_count: Range<usize>, linear_part_size: Range<usize>) -> String {
    (0..random_range(linear_part_count))
        .map(|_| generate(random_range(linear_part_size.clone())))
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
        Command::RustRegex {
            linear_regexes,
            linear_conditional_regexes,
            haystack_size,
            linear_part_count,
            linear_part_size,
        } => {
            let regexes = (0..linear_regexes)
                .map(|_| random_regex_linear(linear_part_count.clone(), linear_part_size.clone()))
                .chain((0..linear_conditional_regexes).map(|_| random_regex_linear_conditional()))
                .collect_vec();
            let haystack = generate(haystack_size);
            let regex_set = RegexSet::new(&regexes).unwrap();
            // duration(|| {
            //     let matches = regex_set.matches(&haystack);
            //     dbg!(matches.iter().count());
            // });

            let strex_set = StrexSet::new(&regexes);
            duration(|| {
                let matches = strex_set.matches(&haystack);
                dbg!(matches.count());
            });

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
