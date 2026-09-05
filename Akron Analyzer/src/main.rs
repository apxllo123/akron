use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let input = std::env::args_os().nth(1).map(PathBuf::from);

    match input {
        Some(path) => {
            let report = akron_analyzer::scanner::analyze_game(&path)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        None => {
            eprintln!("Usage: akron-analyzer <game-directory>");
            std::process::exit(2);
        }
    }

    Ok(())
}
