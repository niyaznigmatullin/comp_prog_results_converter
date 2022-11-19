use crate::standings::contest::{Contest, Contestant, Run, Verdict};
use html5ever::LocalName;
use scraper::{ElementRef, Html, Selector};
use selectors::attr::CaseSensitivity;
use selectors::Element;
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;

trait Finder {
    type OutputType;

    fn find_first(self, tag: &str) -> Option<Self::OutputType>;

    fn find_all(self, tag: &str) -> Vec<Self::OutputType>;

    fn get_text(self) -> String;

    fn convert_text<T>(self) -> T
    where
        T: FromStr,
        <T as FromStr>::Err: Debug;
}

impl Finder for ElementRef<'_> {
    type OutputType = Self;

    fn find_first(self, tag: &str) -> Option<Self> {
        self.select(&Selector::parse(tag).unwrap()).next()
    }

    fn find_all(self, tag: &str) -> Vec<Self> {
        self.select(&Selector::parse(tag).unwrap()).collect()
    }

    fn get_text(self) -> String {
        self.text().collect::<String>()
    }

    fn convert_text<T>(self) -> T
    where
        T: FromStr,
        <T as FromStr>::Err: Debug,
    {
        self.get_text().trim().parse::<T>().unwrap()
    }
}

fn has_class(element: &ElementRef, class_name: &str) -> bool {
    element.has_class(
        &LocalName::from(class_name),
        CaseSensitivity::AsciiCaseInsensitive,
    )
}

fn is_header2(element: &ElementRef) -> bool {
    has_class(element, "table2-header")
}

fn is_solved(element: &ElementRef) -> bool {
    has_class(element, "solved") || has_class(element, "first")
}
pub(crate) fn parse_kattis(s: String) -> Contest {
    let document = Html::parse_document(&s);
    let table = document
        .select(&Selector::parse("table.standings-table").unwrap())
        .next()
        .unwrap();
    let problem_names = get_problems(&table);
    let mut teams = HashMap::new();
    let runs = {
        let rows = get_standings_rows(&table, 6 + problem_names.len());
        let mut runs = rows
            .into_iter()
            .map(|row| {
                let columns = row.find_all("td").into_iter().collect::<Vec<_>>();
                if columns.is_empty() || columns.iter().any(|cell| is_header2(cell)) {
                    return Vec::new();
                }
                let name = get_name_from_cell(&columns[1]);
                let id = teams.len().to_string();
                teams.insert(
                    id.clone(),
                    Contestant {
                        id: id.clone(),
                        name,
                    },
                );
                // let solved = columns[4].convert_text::<usize>();
                // let penalty = columns[5].convert_text::<usize>();
                parse_runs_for_team(columns, id)
            })
            .flatten()
            .collect::<Vec<_>>();
        runs.sort_by_key(|run| run.time);
        runs
    };
    Contest {
        contestants: teams,
        runs,
        problem_names,
    }
}

fn get_name_from_cell(cell: &ElementRef) -> String {
    cell.find_first("div")
        .unwrap()
        .get_text()
        .trim()
        .to_string()
}

fn parse_runs_for_team(columns: Vec<ElementRef>, id: String) -> Vec<Run> {
    columns
        .into_iter()
        .skip(6)
        .zip(0..)
        .map(|(cell, problem_id)| {
            let solved = is_solved(&cell);
            let attempts = get_attempts_from_cell(&cell);
            let time = if solved {
                get_time_from_cell(&cell)
            } else {
                300
            };
            let mut runs = Vec::new();
            for attempt in 0..attempts {
                let verdict = if !solved || attempt + 1 < attempts {
                    Verdict::REJECTED
                } else {
                    Verdict::ACCEPTED
                };
                let time = if attempt + 1 < attempts {
                    time.saturating_sub(5)
                } else {
                    time
                } * 60;
                runs.push(Run {
                    contestant_id: id.clone(),
                    problem_id,
                    verdict,
                    time,
                    attempt,
                })
            }
            runs
        })
        .flatten()
        .collect::<Vec<_>>()
}

fn get_attempts_from_cell(cell: &ElementRef) -> usize {
    cell.find_first(".standings-table-result-cell-text")
        .and_then(|span| parse_first_token::<usize>(span))
        .unwrap_or(0)
}

fn get_time_from_cell(cell: &ElementRef) -> usize {
    cell.find_first(".standings-table-result-cell-time")
        .and_then(|span| parse_first_token::<usize>(span))
        .unwrap()
}

fn get_standings_rows<'a>(table: &'a ElementRef, expect_columns: usize) -> Vec<ElementRef<'a>> {
    if let Some(body) = table.find_first("tbody") {
        body.find_all("tr")
            .into_iter()
            .filter(|row| !is_header2(row) && row.find_all("td").len() >= expect_columns)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    }
}

fn get_problems(table: &ElementRef) -> Vec<String> {
    let row = table
        .find_first("thead")
        .and_then(|head| head.find_first("tr"))
        .unwrap();
    row.find_all("th")
        .into_iter()
        .skip(3)
        .map(|element| element.text().collect::<String>())
        .collect::<Vec<_>>()
}

fn parse_first_token<T: FromStr>(element: ElementRef) -> Option<T> {
    element
        .get_text()
        .split(|c: char| c.is_whitespace())
        .skip_while(|s| s.is_empty())
        .next()
        .and_then(|token| token.parse().ok())
}
