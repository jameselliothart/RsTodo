# RsTodo

## Summary

A command line task tracking and reporting tool demonstrating both file and SQLite persistence.

## Usage

### done

- (A)dd completed tasks: `$ done a 'some completed task'`
- Query completed tasks
    - Completed (d)ays ago: `$ done d 2`
    - Completed  (w)eeks ago: `$ done w 1`

## todo (Coming Soon)

- (A)dd tasks todo: `$ todo a 'implement this feature'`
- (R)emove todos and persist them as completed items (query with `done`): `$ todo r 0`
- List all todo items: `$ todo`

## See Also

The original version of this application was written in PowerShell ([pstodo](https://github.com/jameselliothart/pstodo)) for personal use.
It is useful as a [code kata](https://jameselliothart.github.io/todo-kata-introduction) to implement in multiple languages due to its simple but not trivial "business" logic and the opportunity to leverage both file and database I/O.
