//! Sudoku solver using constraint propagation
//! Inspired by https://norvig.com/sudoku.html

mod cpu_solver;
mod gpu_solver;
mod sudoku_board;

use cpu_solver::cpu_solve_boards;
use gpu_solver::gpu_solve_boards;
use sudoku_board::{EASY_EXAMPLE, HARD_EXAMPLE};

// run on cpu
async fn run_cpu() {
    println!("Initial Board");
    println!("{}", HARD_EXAMPLE);

    let t1 = std::time::Instant::now();
    let results = cpu_solve_boards(&vec![HARD_EXAMPLE]);
    println!("CPU RUNTIME: {}ms", t1.elapsed().as_millis());

    for maybe_solved in results {
        if let Some(solved) = maybe_solved {
            println!("Solved Board");
            println!("{}", solved);
        } else {
            println!("Failed to Solve");
        }
    }

    println!("Done");
}

// run on gpu
async fn run_gpu() {
    println!("Initial Board");
    println!("{}", EASY_EXAMPLE);

    let t1 = std::time::Instant::now();
    let results = gpu_solve_boards(&vec![EASY_EXAMPLE]).await;
    println!("GPU RUNTIME: {}ms", t1.elapsed().as_millis());

    for maybe_solved in results {
        if let Some(solved) = maybe_solved {
            println!("Solved Board");
            println!("{}", solved);
        } else {
            println!("Failed to Solve");
        }
    }

    println!("Done");
}

// main fn
pub fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        let gpu_mode = if let Some(flag) = std::env::args().nth(1) {
            flag == "gpu"
        } else { false };
        if gpu_mode {
            pollster::block_on(run_gpu());
        } else {
            pollster::block_on(run_cpu());
        }
    }
}
