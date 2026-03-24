use crate::cli::run;
use simply_colored::*;

mod cli;
mod list;

fn main() -> anyhow::Result<()> {
    if let Err(e) = run() {
        println!("{RED}{BOLD}Error occurred: {NO_BOLD}{}", e)
    }
    Ok(())
}
