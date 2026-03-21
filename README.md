# tsk

tsk is a simple task manager that saves your TODOs in a human-readable markdown
file.

We have two modes:

- Command-based: you can add/list/remove tasks by calling commands
- TUI: you can manage tasks using a TUI

## Markdown file structure

It ignores all lines that don't start with an arbitrary amount of whitespaces
and `- [`.

One line = one task with the following format:

```txt
- [ ] <ID>: <NAME>
or
- [x] <ID>: <NAME>
or
- [x] <PARENT_ID>/<ID>: <NAME>
```

The caveat is that the program just overrides the file with its own formatting
when modifying the list so everything that is not a task gets deleted upon any
mutation.
