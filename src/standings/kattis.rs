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

pub(crate) fn parse_kattis(s: String) -> Contest {
    let document = Html::parse_document(&s);
    let table = document
        .select(&Selector::parse("table").unwrap())
        .next()
        .unwrap();
    let problem_names = {
        let row = table
            .find_first("thead")
            .and_then(|head| head.find_first("tr"))
            .unwrap();
        dbg!(row.inner_html());
        row.find_all("th")
            .into_iter()
            .skip(3)
            .map(|element| element.text().collect::<String>())
            .collect::<Vec<_>>()
    };
    let mut teams = HashMap::new();
    let runs = {
        let rows = table
            .find_first("tbody")
            .map(|body| body.find_all("tr"))
            .unwrap_or(Vec::new());
        let mut runs = rows
            .into_iter()
            .filter(|row| {
                !row.has_class(
                    &LocalName::from("table2-header"),
                    CaseSensitivity::AsciiCaseInsensitive,
                )
            })
            .map(|row| {
                let columns = row.find_all("td").into_iter().collect::<Vec<_>>();
                let name = columns[1]
                    .find_first("div")
                    .unwrap()
                    .get_text()
                    .trim()
                    .to_string();
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
                columns
                    .into_iter()
                    .skip(6)
                    .zip(0..)
                    .map(|(cell, problem_id)| {
                        let solved = cell.has_class(
                            &LocalName::from("solved"),
                            CaseSensitivity::AsciiCaseInsensitive,
                        );
                        let attempts = cell
                            .find_first(".standings-table-result-cell-text")
                            .and_then(|span| parse_first_token::<usize>(span))
                            .unwrap_or(0);
                        let time = if solved {
                            cell.find_first(".standings-table-result-cell-time")
                                .and_then(|span| parse_first_token::<usize>(span))
                                .unwrap()
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

fn parse_first_token<T: FromStr>(element: ElementRef) -> Option<T> {
    element
        .get_text()
        .split(|c: char| c.is_whitespace())
        .skip_while(|s| s.is_empty())
        .next()
        .and_then(|token| token.parse().ok())
}
