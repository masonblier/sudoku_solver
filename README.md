sudoku_solver
===
Simple Sudoku Solver using Constraint Propagation, inspired by https://norvig.com/sudoku.html

CPU implementation in Rust, and GPU compute implementation using WGSL

Run
---
`cargo run `
```
Initial Board
. . . 7 . 4 . 2 .
. . . . . . 9 . 6
. 4 . . . 8 . . .
. 5 . . . . 2 . .
. 7 . 3 9 . . 5 .
. . 4 . . . . 6 1
. . 1 . 8 . . . 3
. . 8 . . . . . .
5 2 . . . . 4 . .
CPU RUNTIME: 7424ms
Solved Board
9 1 5 7 6 4 3 2 8
2 8 7 1 3 5 9 4 6
6 4 3 9 2 8 1 7 5
8 5 6 4 7 1 2 3 9
1 7 2 3 9 6 8 5 4
3 9 4 8 5 2 7 6 1
4 6 1 2 8 7 5 9 3
7 3 8 5 4 9 6 1 2
5 2 9 6 1 3 4 8 7
Done
```

`cargo run -- gpu`
```
Initial Board
. . . 7 . 4 . 2 .
. . . . . . 9 . 6
. 4 . . . 8 . . .
. 5 . . . . 2 . .
. 7 . 3 9 . . 5 .
. . 4 . . . . 6 1
. . 1 . 8 . . . 3
. . 8 . . . . . .
5 2 . . . . 4 . .

GPU RUNTIME: 3410ms
Solved Board
9 1 5 7 6 4 3 2 8
2 8 7 1 3 5 9 4 6
6 4 3 9 2 8 1 7 5
8 5 6 4 7 1 2 3 9
1 7 2 3 9 6 8 5 4
3 9 4 8 5 2 7 6 1
4 6 1 2 8 7 5 9 3
7 3 8 5 4 9 6 1 2
5 2 9 6 1 3 4 8 7
Done
```
