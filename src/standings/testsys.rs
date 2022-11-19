use crate::standings::contest::{Contest, Verdict};
use std::fmt::Write;

pub(crate) fn write_dat(contest: Contest) -> String {
    let mut result = String::new();
    writeln!(result, "@problems {}", contest.problem_names.len()).unwrap();
    writeln!(result, "@teams {}", contest.contestants.len()).unwrap();
    writeln!(result, "@submissions {}", contest.runs.len()).unwrap();
    for (problem_name, id) in contest.problem_names.iter().zip(0..) {
        let letter = char::from(b'A' + id);
        writeln!(result, "@p {},{},20,0", letter, problem_name).unwrap();
    }
    for contestant in contest.contestants.values() {
        writeln!(result, "@t {},0,1,{}", contestant.id, contestant.name).unwrap();
    }
    for run in contest.runs {
        let problem_name = contest.problem_names[run.problem_id].clone();
        let verdict = match run.verdict {
            Verdict::ACCEPTED => "OK",
            _ => "RJ",
        };
        writeln!(
            &mut result,
            "@s {},{},{},{},{}",
            run.contestant_id,
            problem_name,
            run.attempt + 1,
            run.time,
            verdict
        )
        .unwrap();
    }
    result
}
