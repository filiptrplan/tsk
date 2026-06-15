use std::fmt::format;
use std::io;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier};
use ratatui::text::Text;
use ratatui::widgets::{List as TuiList, ListState};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::Rect,
    widgets::{Paragraph, Widget},
};

use crate::cli::save_list_to_disk;
use crate::list::{Status, Task, TaskPatch};
use crate::{cli::read_list_from_md, list::List};

struct ListFrame {
    list_state: ListState,
    task_ids: Vec<u16>,
}

impl ListFrame {
    fn new(task_ids: Vec<u16>, preselect: bool) -> Self {
        let list_state = if preselect {
            ListState::default().with_selected(Some(0))
        } else {
            ListState::default()
        };
        Self {
            list_state,
            task_ids,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, task_list: &List) {
        let items = task_list
            .all_tasks()
            .iter()
            .filter(|x| self.task_ids.contains(&x.id))
            .map(|x| {
                let status = match x.status {
                    Status::Todo => " ",
                    Status::Done => "✓",
                };
                format!("[{}] {}", status, x.name)
            });
        let list = TuiList::new(items)
            .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}

pub struct App {
    exit: bool,
    list: List,
    frames: Vec<ListFrame>,
    preview_frame: Option<ListFrame>,
}

fn task_vec_to_ids(vec: &Vec<Task>) -> Vec<u16> {
    vec.iter().map(|x| x.id).collect()
}

fn task_slice_to_ids(slice: &[Task]) -> Vec<u16> {
    slice.iter().map(|x| x.id).collect()
}

impl App {
    // TODO: add proper error handling here
    pub fn new() -> Self {
        let list = read_list_from_md().unwrap();
        let tasks = list.toplevel_tasks();
        let mut new_self = Self {
            exit: false,
            list,
            frames: vec![ListFrame::new(task_vec_to_ids(&tasks), true)],
            preview_frame: None,
        };
        new_self.set_preview_frame();
        new_self
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(frame.area());

        let length = self.frames.len();

        // render leftmost frame
        if length > 1
            && let Some(list_frame) = self.frames.get_mut(length - 2)
        {
            list_frame.render(frame, layout[0], &self.list);
        }
        let list_frame = self.frames.last_mut().unwrap();
        list_frame.render(frame, layout[1], &self.list);

        if let Some(preview_frame) = &mut self.preview_frame {
            preview_frame.render(frame, layout[2], &self.list);
        }
    }

    fn current_main_frame_mut(&mut self) -> &mut ListFrame {
        self.frames.last_mut().unwrap()
    }

    fn current_main_frame(&self) -> &ListFrame {
        self.frames.last().unwrap()
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == event::KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn current_selected_task(&self) -> Option<&Task> {
        let main_frame = self.current_main_frame();
        if let Some(idx) = main_frame.list_state.selected() {
            let task_id = main_frame.task_ids.get(idx);
            if let Some(task_id) = task_id {
                self.list.get_task(*task_id).ok()
            } else {
                None
            }
        } else {
            None
        }
    }

    fn add_frame_from_selected(&mut self) {
        let parent_task = self.current_selected_task();
        if let Some(parent_task) = parent_task {
            let tasks = self.list.tasks_of_parent(parent_task.id);
            let new_frame = ListFrame::new(task_vec_to_ids(&tasks), true);
            self.frames.push(new_frame);
            self.set_preview_frame();
        }
    }

    fn set_preview_frame(&mut self) {
        if let Some(parent_task) = self.current_selected_task() {
            let tasks = self.list.tasks_of_parent(parent_task.id);
            if tasks.is_empty() {
                self.preview_frame = None;
                return;
            }
            let new_frame = ListFrame::new(task_vec_to_ids(&tasks), false);
            self.preview_frame = Some(new_frame);
        } else {
            self.preview_frame = None;
        }
    }

    fn remove_frame(&mut self) {
        if self.frames.len() <= 1 {
            return;
        }
        self.frames.pop();
        self.set_preview_frame();
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('j') => {
                self.current_main_frame_mut().list_state.select_next();
                self.set_preview_frame();
            }
            KeyCode::Char('k') => {
                self.current_main_frame_mut().list_state.select_previous();
                self.set_preview_frame();
            }
            KeyCode::Char('l') => {
                self.add_frame_from_selected();
            }
            KeyCode::Char('h') => {
                self.remove_frame();
            }
            KeyCode::Char(' ') => {
                let task = self.current_selected_task();
                if let Some(task) = task {
                    let status = if task.status == Status::Done {
                        Status::Todo
                    } else {
                        Status::Done
                    };

                    let patch = TaskPatch::new(task.id).status(status);
                    let _ = self.list.modify_task(patch);
                    // TODO: handle these errors gracefully?
                    save_list_to_disk(&self.list).unwrap();
                }
            }
            KeyCode::Char('d') => {
                let task = self.current_selected_task();
                if let Some(task) = task {
                    let _ = self.list.remove_task(task.id);
                    save_list_to_disk(&self.list).unwrap();
                }
            }
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
