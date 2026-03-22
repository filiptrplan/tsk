use crate::list::{List, Status, TaskPatch};

#[test]
fn create_new_empty_list() {
    let list = List::new();
    assert!(list.tasks.is_empty());
}

#[test]
fn add_task_to_list_sets_correct_name_and_id() {
    let mut list = List::new();
    let _ = list.add_task("Test", None);
    let task = list.tasks.iter().find(|x| x.name == "Test").unwrap();
    assert_eq!(task.name, "Test");
    assert_eq!(task.id, 1);
}

#[test]
fn add_two_tasks_to_list() {
    let mut list = List::new();
    let _ = list.add_task("Test", None);
    let _ = list.add_task("Test2", None);
    let task = list.tasks.iter().find(|x| x.name == "Test").unwrap();
    let task2 = list.tasks.iter().find(|x| x.name == "Test2").unwrap();
    assert_eq!(task.name, "Test");
    assert_eq!(task2.name, "Test2");
}

#[test]
fn adding_task_with_invalid_parent_returns_error() {
    let mut list = List::new();
    let _ = list.add_task("Test", None);
    let ret = list.add_task("Test2", Some(3));
    assert!(ret.is_err())
}

#[test]
fn parse_simple_file() {
    let res = List::parse_from_md("- [x] 123: abc\n lalalald\n- [ ] 1234: abcd");
    assert!(res.is_ok());
}

#[test]
fn save_simple_file() {
    let mut list = List::new();
    let _ = list.add_task("Test", None);
    let _ = list.add_task("Test2", None);
    let _ = list.modify_task(TaskPatch {
        id: 1,
        parent_id: None,
        name: None,
        status: Some(Status::Done),
    });
    println!("{}", list.save_to_md());
}
