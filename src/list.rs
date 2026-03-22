use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{complete::is_not, tag, take_till},
    character::complete::{anychar, digit1, line_ending, newline, space0},
    combinator::{eof, map, opt, recognize},
    multi::{many_till, many0},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
enum Status {
    Todo,
    Done,
}

/// A single task with its owned children
#[derive(Debug, Clone)]
struct Task {
    id: u16,
    parent_id: Option<u16>,
    name: String,
    status: Status,
}

#[derive(Default)]
struct TaskPatch {
    id: u16,
    parent_id: Option<Option<u16>>,
    name: Option<String>,
    status: Option<Status>,
}

/// The whole task list contained in TSK.md
#[derive(Debug)]
struct List {
    /// All tasks
    tasks: Vec<Task>,
    next_id: u16,
}

impl List {
    /// Parses list from markdown
    pub fn parse_from_md(raw_md_text: &str) -> anyhow::Result<Self> {
        let skip_line = terminated(take_till(|c| c == '\n'), alt((line_ending, eof)));
        let parse_line = alt((
            map(List::parse_line_to_task, |x: Task| {
                println!("task!");
                Some(x)
            }),
            map(skip_line, |_| {
                println!("None!");
                None
            }),
        ));
        let parse_result = many_till(parse_line, eof).parse(raw_md_text);
        match parse_result {
            Err(e) => {
                println!("{:?}", e);
                Err(anyhow::format_err!("Parsing of file failed!"))
            }
            Ok((_, (results, _))) => {
                let task_vec = results.into_iter().flatten().collect();
                Ok(List {
                    tasks: task_vec,
                    next_id: 1,
                })
            }
        }
    }

    fn parse_line_to_task(input: &str) -> IResult<&str, Task> {
        let (rem, _) = space0.parse(input)?;
        let (rem, (_, status, _)) =
            tuple((tag("- ["), alt((tag(" "), tag("x"))), tag("]"))).parse(rem)?;
        let (rem, (first_id, second_id)) = delimited(
            space0,
            tuple((digit1, opt(preceded(tag("/"), digit1)))),
            (tag(":"), space0),
        )
        .parse(rem)?;
        let parent_id;
        let parsed_id;
        match second_id {
            None => {
                parsed_id = first_id.parse().unwrap();
                parent_id = None;
            }
            Some(second_id) => {
                parsed_id = second_id.parse().unwrap();
                parent_id = Some(first_id.parse().unwrap());
            }
        }
        let (rem, (name, _)) = many_till(anychar, alt((line_ending, eof))).parse(rem)?;
        let parsed_status = if status == " " {
            Status::Todo
        } else {
            Status::Done
        };
        Ok((
            rem,
            Task {
                status: parsed_status,
                parent_id,
                id: parsed_id,
                name: name.iter().collect(),
            },
        ))
    }

    /// Outputs a markdown file with an optional "original" where it modifies what needs to be
    /// modified
    fn save_to_md(&self) -> String {
        unimplemented!()
    }

    fn new() -> Self {
        List {
            tasks: Vec::new(),
            next_id: 1,
        }
    }

    fn add_task(&mut self, name: &str, parent_id: Option<u16>) -> anyhow::Result<()> {
        if let Some(parent_id) = parent_id
            && !self.tasks.iter().any(|x| x.id == parent_id)
        {
            return Err(anyhow::format_err!("Parent ID {} not found", parent_id));
        }

        self.tasks.push(Task {
            id: self.next_id,
            parent_id,
            name: name.to_owned(),
            status: Status::Todo,
        });

        self.next_id += 1;

        Ok(())
    }

    fn remove_task(&mut self, id: u16) -> anyhow::Result<()> {
        let task_idx = self.tasks.iter().position(|x| x.id == id);
        match task_idx {
            None => {
                return Err(anyhow::format_err!("Task ID {} doesn't exist.", id));
            }
            Some(idx) => {
                self.tasks.remove(idx);
            }
        }
        Ok(())
    }

    fn list_tasks(&self) -> &[Task] {
        &self.tasks
    }

    fn modify_task(&mut self, patch: TaskPatch) -> anyhow::Result<()> {
        let task_idx = self.tasks.iter().position(|x| x.id == patch.id);
        match task_idx {
            None => {
                return Err(anyhow::format_err!("Task ID {} doesn't exist.", patch.id));
            }
            Some(idx) => {
                let task = self.tasks.get_mut(idx).unwrap();
                if let Some(name) = patch.name {
                    task.name = name;
                }
                if let Some(status) = patch.status {
                    task.status = status;
                }
                if let Some(parent_id) = patch.parent_id {
                    task.parent_id = parent_id;
                }
            }
        }

        Ok(())
    }

    fn get_task(&self, id: u16) -> anyhow::Result<&Task> {
        let task = self.tasks.iter().find(|x| x.id == id);
        if task.is_none() {
            return Err(anyhow::format_err!("Task ID {} doesn't exist.", id));
        }
        Ok(task.unwrap())
    }
}
