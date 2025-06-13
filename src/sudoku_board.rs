//! Sudoku solver board data type
//! and helper functions
use std::fmt;

// zero used as None value
#[derive(Clone,Default)]
pub struct SudokuBoard {
    pub values: [[u32; 9]; 9]
}

impl fmt::Display for SudokuBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = self.values.iter().map(|row| {
            row.iter().map(|&cell|
                if cell == 0 { ".".to_string() } else { cell.to_string() })
            .collect::<Vec<String>>().join(" ")
        })
        .collect::<Vec<String>>()
        .join("\n");
        write!(f, "{}", str)
    }
}

pub const EASY_EXAMPLE: SudokuBoard = SudokuBoard {
    values: [
        [ 0, 0, 0, 7, 0, 4, 0, 2, 0],
        [ 0, 0, 0, 0, 0, 0, 9, 0, 6],
        [ 0, 4, 0, 0, 0, 8, 0, 0, 0],
        [ 0, 5, 0, 0, 0, 0, 2, 0, 0],
        [ 0, 7, 0, 3, 9, 0, 0, 5, 0],
        [ 0, 0, 4, 0, 0, 0, 0, 6, 1],
        [ 0, 0, 1, 0, 8, 0, 0, 0, 3],
        [ 0, 0, 8, 0, 0, 0, 0, 0, 0],
        [ 5, 2, 0, 0, 0, 0, 4, 0, 0],
    ]
};

pub const HARD_EXAMPLE: SudokuBoard = SudokuBoard {
    values: [
        [ 4, 0, 0, 0, 0, 0, 8, 0, 5],
        [ 0, 3, 0, 0, 0, 0, 0, 0, 0],
        [ 0, 0, 0, 7, 0, 0, 0, 0, 0],
        [ 0, 2, 0, 0, 0, 0, 0, 6, 0],
        [ 0, 0, 0, 0, 8, 0, 4, 0, 0],
        [ 0, 0, 0, 0, 1, 0, 0, 0, 0],
        [ 0, 0, 0, 6, 0, 3, 0, 7, 0],
        [ 5, 0, 0, 2, 0, 0, 0, 0, 0],
        [ 1, 0, 4, 0, 0, 0, 0, 0, 0],
    ]
};
