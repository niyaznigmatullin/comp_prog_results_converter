
pub struct Contestant {
    pub(crate) id: String,
    pub(crate) name: String,
}
//
// pub struct StandingsRow {
//     contestant_id: String,
//     solved: usize,
//     penalty: usize,
//     problem_results: Vec<ProblemResult>,
// }

#[derive(Debug)]
pub enum Verdict {
    Accepted,
    Rejected,
    CompilationError,
}

#[cfg(test)]
impl Verdict {
    pub(crate) fn is_rejected(&self) -> bool {
        matches!(self, Self::Rejected)
    }

    pub(crate) fn is_accepted(&self) -> bool {
        matches!(self, Self::Accepted)
    }
}

// pub struct ProblemResult {
//     attempts: usize,
//     time: Option<usize>,
//     success: Verdict,
// }

#[derive(Debug)]
pub(crate) struct Run {
    pub(crate) contestant_id: String,
    pub(crate) problem_id: usize,
    pub(crate) verdict: Verdict,
    pub(crate) time: usize,
    pub(crate) attempt: usize,
}

pub(crate) struct Contest {
    pub(crate) contestants: Vec<Contestant>,
    // standings: HashMap<String, StandingsRow>,
    pub(crate) runs: Vec<Run>,
    pub(crate) problem_names: Vec<String>,
}
