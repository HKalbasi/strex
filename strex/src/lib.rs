use std::collections::{HashMap, HashSet, hash_map::Entry};

use aho_corasick::AhoCorasick;
use strex_parser::StrexHir;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct StepId(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ChainId(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StrexId(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct WordId(usize);

#[derive(Debug, Clone, Copy)]
enum WordJob {
    DoMatch {
        id: StrexId,
    },
    StartChain {
        id: ChainId,
        pre_condition: Option<(ChainId, StepId)>,
    },
    StepChain {
        id: ChainId,
        step: StepId,
    },
}

struct Chain {
    sub_chains: Vec<ChainId>,
    final_step: StepId,
    result: WordJob,
}

#[derive(Default)]
struct StrexSetBuilder {
    count: usize,
    chains: Vec<Chain>,
    words: Vec<String>,
    word_to_id: HashMap<String, WordId>,
    word_jobs: Vec<Vec<WordJob>>,
}

impl StrexSetBuilder {
    fn add_strex(&mut self, strex: StrexHir) {
        let match_id = StrexId(self.count);
        self.count += 1;
        self.add_strex_base(strex, WordJob::DoMatch { id: match_id });
    }

    fn word_id(&mut self, word: String) -> WordId {
        match self.word_to_id.entry(word.clone()) {
            Entry::Occupied(occupied_entry) => *occupied_entry.get(),
            Entry::Vacant(vacant_entry) => {
                let id = WordId(self.words.len());
                vacant_entry.insert(id);
                self.words.push(word);
                self.word_jobs.push(vec![]);
                id
            }
        }
    }

    fn add_strex_base(&mut self, strex: StrexHir, result: WordJob) {
        match strex {
            StrexHir::Literal(lit) => {
                let word_id = self.word_id(lit);
                self.word_jobs[word_id.0].push(result);
            }
            StrexHir::Or(strex_hirs) => {
                for strex in strex_hirs {
                    self.add_strex_base(strex, result);
                }
            }
            StrexHir::Concat(strex_hirs) => {
                let (first, rest) = strex_hirs.split_first().unwrap();
                let chain = Chain {
                    sub_chains: vec![],
                    final_step: StepId(rest.len()),
                    result,
                };
                let id = self.add_chain(chain);
                self.add_strex_base(
                    first.clone(),
                    WordJob::StartChain {
                        id,
                        pre_condition: None,
                    },
                );
                let mut step = StepId(1);
                for elem in rest {
                    if *elem == StrexHir::Wild {
                        continue;
                    }
                    self.add_strex_base(elem.clone(), WordJob::StepChain { id, step });
                    step.0 += 1;
                }
            }
            StrexHir::Wild => panic!("Strex matches things unconditionally"),
        }
    }

    fn add_chain(&mut self, chain: Chain) -> ChainId {
        let id = ChainId(self.chains.len());
        self.chains.push(chain);
        id
    }
}

pub struct StrexSet {
    aho: AhoCorasick,
    chains: Vec<Chain>,
    word_jobs: Vec<Vec<WordJob>>,
}

struct ChainState<'a> {
    strex_set: &'a StrexSet,
    matches: HashSet<StrexId>,
    states: HashMap<ChainId, StepId>,
}

impl ChainState<'_> {
    fn kill_chain(&mut self, chain: ChainId) {
        if self.states.remove(&chain).is_some() {
            for sub_chain in &self.strex_set.chains[chain.0].sub_chains {
                self.kill_chain(*sub_chain);
            }
        }
    }

    fn do_step(&mut self, chain: ChainId) {
        let step = self.states.get_mut(&chain).unwrap();
        step.0 += 1;
        if *step == self.strex_set.chains[chain.0].final_step {
            self.kill_chain(chain);
            self.do_word_job(self.strex_set.chains[chain.0].result);
            return;
        }
    }

    fn do_word_job(&mut self, word_job: WordJob) {
        match word_job {
            WordJob::DoMatch { id } => {
                self.matches.insert(id);
            }
            WordJob::StartChain { id, pre_condition } => {
                if let Some((chain, step)) = pre_condition {
                    if self.states.get(&chain) != Some(&step) {
                        return;
                    }
                }
                self.states.insert(id, StepId(1));
            }
            WordJob::StepChain { id, step } => {
                if self.states.get(&id) == Some(&step) {
                    self.do_step(id);
                }
            }
        }
    }
}

impl StrexSet {
    pub fn new<I, S>(strexes: I) -> Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        let mut builder = StrexSetBuilder::default();

        for strex in strexes {
            builder.add_strex(StrexHir::parse(strex.as_ref()).unwrap());
        }

        Self {
            aho: AhoCorasick::new(builder.words).unwrap(),
            chains: builder.chains,
            word_jobs: builder.word_jobs,
        }
    }

    fn new_state(&self) -> ChainState<'_> {
        ChainState {
            strex_set: self,
            matches: HashSet::new(),
            states: HashMap::new(),
        }
    }

    pub fn matches(&self, haystack: &str) -> impl Iterator<Item = StrexId> {
        let mut chain_state = self.new_state();

        for word_match in self.aho.find_iter(haystack) {
            for word_job in &self.word_jobs[word_match.pattern().as_usize()] {
                chain_state.do_word_job(*word_job);
            }
        }

        chain_state.matches.into_iter()
    }
}
