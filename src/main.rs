use crate::standings::kattis::parse_kattis;
use crate::standings::testsys::write_dat;
use std::fs::read_to_string;
use clap::Parser;

mod standings;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // #[arg(short, long)]
    // from: String,
    // #[arg(short, long)]
    // to: String,
    #[arg(long)]
    file: String,
}

fn main() {
    let args = Args::parse();
    let string = read_to_string(args.file).unwrap();
    let contest = parse_kattis(string);
    println!("{}", write_dat(contest));
}
