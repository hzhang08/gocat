mod sgf_parser;
mod game;
mod ui;

use clap::Parser;
use std::fs;
use crate::sgf_parser::parse_sgf;
use crate::game::GoGame;
use crate::ui::run_ui;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the SGF file
    sgf_path: String,
}

fn main() {
    let args = Args::parse();
    let sgf_content = match fs::read_to_string(&args.sgf_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read SGF file: {}", e);
            std::process::exit(1);
        }
    };

    let sgf = match parse_sgf(&sgf_content) {
        Ok(sgf) => sgf,
        Err(e) => {
            eprintln!("Failed to parse SGF file: {}", e);
            std::process::exit(1);
        }
    };

    let mut game = GoGame::new(sgf);
    if let Err(e) = run_ui(&mut game) {
        eprintln!("Error running UI: {}", e);
        std::process::exit(1);
    }
}
