use std::io::Write;
use std::{
    fs::{File, read_to_string},
    path::PathBuf,
};

use clap::{Parser, Subcommand};

use crate::list::{List, Status, TaskPatch};

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
    /// Marks the task as done
    Done(DoneArgs),
    List,
}

#[derive(clap::Args)]
struct AddArgs {
    name: String,
    parent: Option<u16>,
}
#[derive(clap::Args)]
struct DoneArgs {
    id: u16,
}
#[derive(clap::Args)]
struct RemoveArgs {
    id: u16,
}
#[derive(clap::Args)]
struct ModifyArgs {
    id: u16,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    parent: Option<u16>,
    #[arg(long)]
    remove_parent: bool,
}

fn read_list_from_md() -> anyhow::Result<List> {
    let path = PathBuf::from("./TSK.md");
    if !path.exists() {
        return Err(anyhow::format_err!(
            "TSK.md does not exist in current directory."
        ));
    }
    let raw_string = read_to_string(path)?;
    List::parse_from_md(&raw_string)
}

fn save_list_to_disk(list: &List) -> anyhow::Result<()> {
    let path = PathBuf::from("./TSK.md");
    let mut output = File::create(path)?;
    Ok(write!(output, "{}", list.save_to_md())?)
}

fn run_cli(command: &Commands) -> anyhow::Result<()> {
    let mut list = read_list_from_md()?;
    match command {
        Commands::Add(add_args) => {
            list.add_task(&add_args.name, add_args.parent)?;
            save_list_to_disk(&list)?;
        }
        Commands::Remove(remove_args) => {
            let task = list.get_task(remove_args.id)?.name.clone();
            list.remove_task(remove_args.id)?;
            save_list_to_disk(&list)?;
            println!("Removed task '{}'", task);
        }
        Commands::Modify(modify_args) => {
            let parent = if modify_args.remove_parent {
                Some(None)
            } else {
                modify_args.parent.map(Some)
            };
            let patch = TaskPatch {
                id: modify_args.id,
                parent_id: parent,
                name: modify_args.name.clone(),
                status: None,
            };
            list.modify_task(patch)?;
            save_list_to_disk(&list)?;
            let task = list.get_task(modify_args.id)?.name.clone();
            println!("Modified task '{}'", task);
        }
        Commands::Done(done_args) => {
            let patch = TaskPatch {
                id: done_args.id,
                parent_id: None,
                name: None,
                status: Some(Status::Done),
            };
            list.modify_task(patch)?;
            save_list_to_disk(&list)?;
            let task = list.get_task(done_args.id)?.name.clone();
            println!("Set task as done: '{}'", task);
        }
        Commands::List => {
            println!("{}", list);
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
