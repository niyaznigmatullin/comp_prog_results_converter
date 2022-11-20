use crate::standings::contest::{Contest, Contestant, Run, Verdict};
use std::collections::HashMap;
use std::fmt::Write;

pub(crate) fn write_dat(contest: Contest) -> String {
    let mut result = String::new();
    writeln!(result, "@problems {}", contest.problem_names.len()).unwrap();
    writeln!(result, "@teams {}", contest.contestants.len()).unwrap();
    writeln!(result, "@submissions {}", contest.runs.len()).unwrap();
    result.push_str(&write_problems(&contest.problem_names));
    let team_ids = assign_teams_integer_ids(&contest.contestants);
    result.push_str(&write_teams(&contest.contestants, &team_ids));
    result.push_str(&write_runs(&contest.runs, &team_ids));
    result
}

fn write_runs(runs: &Vec<Run>, team_ids: &HashMap<String, u32>) -> String {
    let mut result = String::new();
    for run in runs {
        let letter = char::from(b'A' + run.problem_id as u8);
        let verdict = match run.verdict {
            Verdict::Accepted => "OK",
            Verdict::Rejected => "RJ",
            Verdict::CompilationError => "CE",
        };
        writeln!(
            result,
            "@s {},{},{},{},{}",
            team_ids.get(&run.contestant_id).unwrap(),
            letter,
            run.attempt + 1,
            run.time,
            verdict
        )
        .unwrap();
    }
    result
}

fn write_teams(teams: &Vec<Contestant>, team_ids: &HashMap<String, u32>) -> String {
    let mut result = String::new();
    for contestant in teams {
        writeln!(
            result,
            "@t {},0,1,{}",
            team_ids.get(&contestant.id).unwrap(),
            contestant.name
        )
        .unwrap();
    }
    result
}

fn assign_teams_integer_ids(contestants: &Vec<Contestant>) -> HashMap<String, u32> {
    let mut team_ids = HashMap::new();
    for (contestant, id) in contestants.iter().zip(0..) {
        team_ids.insert(contestant.id.clone(), id);
    }
    team_ids
}

fn write_problems(problem_names: &Vec<String>) -> String {
    let mut result = String::new();
    for (problem_name, id) in problem_names.iter().zip(0..) {
        let letter = char::from(b'A' + id);
        writeln!(result, "@p {},{},20,0", letter, problem_name).unwrap();
    }
    result
}

pub(crate) fn read_dat(data: String) -> Contest {
    let teams = read_teams(&data);
    let problems = read_problems(&data);
    let mut problems = problems.into_iter().collect::<Vec<_>>();
    problems.sort_by_key(|(id, _)| id.to_string());
    let (ids, problems): (Vec<_>, Vec<_>) = problems.into_iter().unzip();
    let problems_id_mapping = ids
        .into_iter()
        .zip(0..)
        .map(|(old_id, new_id)| (old_id, new_id))
        .collect::<HashMap<_, _>>();
    let runs = read_submissions(&data, &problems_id_mapping);
    Contest {
        contestants: teams,
        problem_names: problems,
        runs,
    }
}

fn read_submissions(data: &str, problem_id: &HashMap<String, usize>) -> Vec<Run> {
    data.lines()
        .filter(|line| line.starts_with("@s") && !line.starts_with("@sub"))
        .map(|line| {
            let tokens = read_tokens(line, 5);
            Run {
                contestant_id: tokens[0].to_string(),
                problem_id: *problem_id.get(tokens[1]).unwrap(),
                attempt: tokens[2].parse::<usize>().unwrap() - 1,
                time: tokens[3].parse().unwrap(),
                verdict: match tokens[4] {
                    "OK" => Verdict::Accepted,
                    "RJ" => Verdict::Rejected,
                    "CE" => Verdict::CompilationError,
                    _ => Verdict::Rejected,
                },
            }
        })
        .collect()
}

fn read_problems(data: &str) -> HashMap<String, String> {
    let mut problems = HashMap::new();
    for line in data
        .lines()
        .filter(|line| line.starts_with("@p") && !line.starts_with("@probl"))
    {
        let tokens = read_tokens(line, 4);
        problems.insert(tokens[0].to_string(), tokens[1].to_string());
    }
    problems
}

fn read_teams(data: &str) -> Vec<Contestant> {
    data.lines()
        .filter(|line| line.starts_with("@t") && !line.starts_with("@teams"))
        .map(|line| {
            let tokens = read_tokens(line, 4);
            Contestant {
                id: tokens[0].to_string(),
                name: tokens[3].to_string(),
            }
        })
        .collect()
}

fn read_tokens(line: &str, count: usize) -> Vec<&str> {
    get_second_part(line).splitn(count, ',').collect::<Vec<_>>()
}

fn get_second_part(line: &str) -> &str {
    line.splitn(2, |c: char| c.is_whitespace())
        .skip(1)
        .next()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use crate::standings::contest::Contest;
    use crate::standings::tests::tests::read_resource;
    use crate::standings::testsys::{read_dat, write_dat};
    use std::collections::HashMap;

    fn make_standings(contest: &Contest) -> Vec<(String, usize, usize)> {
        struct Team {
            name: String,
            solved: usize,
            penalty: usize,
            problem_result: Vec<(usize, bool)>,
        }
        let problems_count = contest.problem_names.len();
        let mut teams = contest
            .contestants
            .iter()
            .map(|c| {
                let team = Team {
                    name: c.name.to_string(),
                    solved: 0,
                    penalty: 0,
                    problem_result: vec![(0, false); problems_count],
                };
                (c.id.to_string(), team)
            })
            .collect::<HashMap<_, _>>();
        for x in &contest.runs {
            let team = teams.get_mut(&x.contestant_id).unwrap();
            let result = &mut team.problem_result[x.problem_id];
            if !result.1 {
                if x.verdict.is_rejected() {
                    result.0 += 1;
                } else if x.verdict.is_accepted() {
                    team.solved += 1;
                    team.penalty += x.time + 20 * 60 * result.0;
                    result.1 = true;
                }
            }
        }
        let mut teams = teams
            .into_values()
            .map(|team| (team.name, team.solved, team.penalty))
            .collect::<Vec<_>>();
        teams.sort_by(|t1, t2| {
            t1.1.cmp(&t2.1)
                .reverse()
                .then_with(|| t1.2.cmp(&t2.2))
                .then_with(|| t1.0.cmp(&t2.0))
        });
        teams
    }

    #[test]
    fn test_parse_and_write_ncpc22() {
        let dat1 = read_resource("ncpc22.dat");
        let contest1 = read_dat(dat1);
        let standings1 = make_standings(&contest1);
        let dat2 = write_dat(contest1);
        let contest2 = read_dat(dat2);
        let standings2 = make_standings(&contest2);
        assert_eq!(standings1, standings2);
    }
}
