use std::{fs::read_to_string, path::PathBuf};

use clap::{Parser, Subcommand};

use crate::list::List;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Add(AddArgs),
    Remove(RemoveArgs),
    Modify(ModifyArgs),
    List,
}

#[derive(clap::Args)]
struct AddArgs {}
#[derive(clap::Args)]
struct RemoveArgs {}
#[derive(clap::Args)]
struct ModifyArgs {}

fn read_list_from_md() -> anyhow::Result<List> {
    let path = PathBuf::from("./TSK.md");
    if !path.exists() {
        return Err(anyhow::format_err!(
            "TSK.md does not exist in current directory."
        ));
    }
    let raw_string = read_to_string(path)?;
    Ok(List::parse_from_md(&raw_string)?)
}

fn run_cli(command: &Commands) -> anyhow::Result<()> {
    let mut list = read_list_from_md()?;
    match command {
        Commands::Add(add_args) => todo!(),
        Commands::Remove(remove_args) => todo!(),
        Commands::Modify(modify_args) => todo!(),
        Commands::List => {
            println!("{:?}", list);
        }
    }
    Ok(())
}

fn run_tui() -> anyhow::Result<()> {
    Ok(())
}

pub fn run() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        None => run_tui()?,
        Some(command) => run_cli(&command)?,
    };

    Ok(())
}
