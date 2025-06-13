//! CPU simple solver
use crate::sudoku_board::{SudokuBoard};

// holds possible remaining moves
#[derive(Clone,Default)]
struct SudokuMoveGroups {
    values: [[Vec<u32>; 9]; 9]
}

// returns updated move set with conflicting moves removed
// or None if move results in cells with no possible moves
fn propagate_move(
    r: usize, c: usize, candidate: u32,
    candidate_moves: &SudokuMoveGroups
) -> Option<SudokuMoveGroups> {
    let mut updated_moves = candidate_moves.clone();
    // remove colliding moves from other groups
    for (cr, row) in candidate_moves.values.iter().enumerate() {
        for (cc, moveset) in row.iter().enumerate() {
            // candidate cell
            if c == cc && r == cr {
                updated_moves.values[r][c] = vec![candidate];
            }
            // check column matches
            else if c == cc {
                let updated_moveset = moveset.clone().into_iter()
                    .filter(|m| *m != candidate).collect::<Vec<u32>>();
                if updated_moveset.len() == 0 {
                    // failed move
                    return None;
                }
                updated_moves.values[cr][cc] = updated_moveset;
            }
            // check row matches
            else if r == cr {
                let updated_moveset = moveset.clone().into_iter()
                    .filter(|m| *m != candidate).collect::<Vec<u32>>();
                if updated_moveset.len() == 0 {
                    // failed move
                    return None;
                }
                updated_moves.values[cr][cc] = updated_moveset;
            }
            // check 3x3 area matches
            else if r / 3 == cr / 3 && c / 3 == cc / 3 {
                let updated_moveset = moveset.clone().into_iter()
                    .filter(|m| *m != candidate).collect::<Vec<u32>>();
                if updated_moveset.len() == 0 {
                    // failed move
                    return None;
                }
                updated_moves.values[cr][cc] = updated_moveset;
            }
        }
    }
    Some(updated_moves)
}

fn try_candidates(moves: &SudokuMoveGroups, (r, c): (usize, usize)) -> Option<SudokuMoveGroups> {
    for candidate in moves.values[r][c].iter() {
        // create next move set with candidate move applied
        if let Some(candidate_moves) = propagate_move(r, c, *candidate, &moves) {
            // try candidate
            let next_move = if c == 8 { (r + 1, 0) } else { (r, c + 1) };
            if next_move.0 > 8 { return Some(candidate_moves) };
            if let Some(solved_move_group) = try_candidates(&candidate_moves, next_move) {
                return Some(solved_move_group)
            }
        }
    }
    None
}

fn solve_board(board: &SudokuBoard) -> Option<SudokuBoard> {
    // generate move groups from board
    let mut moves = SudokuMoveGroups::default();
    for (r, row) in board.values.iter().enumerate() {
        for (c, &cell) in row.iter().enumerate() {
            if cell == 0 {
                moves.values[r][c] = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
            } else {
                moves.values[r][c] = vec![cell];
            }
        }
    }

    // try to get next solved candidate
    if let Some(solved_move_group) = try_candidates(&moves, (0, 0)) {
        // convert moveset back to board
        let mut solved_board = SudokuBoard::default();
        for (r, row) in solved_move_group.values.iter().enumerate() {
            for (c, moveset) in row.into_iter().enumerate() {
                if moveset.len() == 1 {
                    solved_board.values[r][c] = moveset[0]
                } else {
                    panic!("Too many remaining moves");
                }
            }
        }
        Some(solved_board)
    } else {
        None
    }
}

pub fn cpu_solve_boards(boards: &Vec<SudokuBoard>) -> Vec<Option<SudokuBoard>> {
    boards.iter().map(solve_board).collect()
}
