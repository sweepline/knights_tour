use rayon::prelude::*;

const BOARD_WIDTH: u8 = 6;
const BOARD_SIZE: usize = (BOARD_WIDTH * BOARD_WIDTH) as usize;

// We can jump to 8 locations with a knight.
// JUMPS would then be the index in the board array for each jump.
// The value will be u8::MAX if the location cannot be jumped to.
// Jumps are in the order of
// + 0 + 1 +
// 2 + + + 3
// + + K + +
// 4 + + + 5
// + 6 + 7 +
type Jumps = [u8; 8];

const fn find_jump(pos: u8, jump: u8) -> u8 {
    if pos < BOARD_WIDTH * 2 && jump <= 1 {
        // Top two rows
        return u8::MAX;
    }
    if pos < BOARD_WIDTH && jump <= 3 {
        // Top row
        return u8::MAX;
    }
    if pos >= (BOARD_SIZE as u8 - 2 * BOARD_WIDTH) && jump >= 6 {
        // Bottom two rows
        return u8::MAX;
    }
    if pos >= (BOARD_SIZE as u8 - BOARD_WIDTH) && jump >= 4 {
        // Bottom row
        return u8::MAX;
    }
    if pos % BOARD_WIDTH >= (BOARD_WIDTH - 2) && (jump == 3 || jump == 5) {
        // Rightmost two rows
        return u8::MAX;
    }
    if pos % BOARD_WIDTH >= (BOARD_WIDTH - 1) && (jump == 1 || jump == 7) {
        // Rightmost row
        return u8::MAX;
    }
    if pos % BOARD_WIDTH <= 1 && (jump == 2 || jump == 4) {
        // Leftmost two rows
        return u8::MAX;
    }
    if pos % BOARD_WIDTH <= 0 && (jump == 0 || jump == 6) {
        // Leftmost row
        return u8::MAX;
    }

    match jump {
        0 => pos - BOARD_WIDTH * 2 - 1,
        1 => pos - BOARD_WIDTH * 2 + 1,
        2 => pos - BOARD_WIDTH - 2,
        3 => pos - BOARD_WIDTH + 2,
        4 => pos + BOARD_WIDTH - 2,
        5 => pos + BOARD_WIDTH + 2,
        6 => pos + BOARD_WIDTH * 2 - 1,
        7 => pos + BOARD_WIDTH * 2 + 1,
        _ => panic!("Jump to invalid location"),
    }
}

const fn make_possible_jumps() -> [Jumps; BOARD_SIZE] {
    let mut jumps: [Jumps; BOARD_SIZE] = [[0, 0, 0, 0, 0, 0, 0, 0]; BOARD_SIZE];
    let mut i: u8 = 0;
    while i < BOARD_SIZE as u8 {
        let mut k: u8 = 0;
        while k < 8 {
            jumps[i as usize][k as usize] = find_jump(i, k);
            k += 1;
        }
        i += 1;
    }
    jumps
}
const POSSIBLE_JUMPS: [Jumps; BOARD_SIZE] = make_possible_jumps();

// A thread needs to keep locations that have been jumped to,
// and a stack of where we have jumped to and what possible jumps we have tried.
type Visited = [bool; BOARD_SIZE];
type Stack = [(u8, u8); BOARD_SIZE];

fn main() {
    // Now instead parallelize on only start 0, to find closed tours faster.
    // Means we BFS until we have threads amount of prefix paths. Then do DFS until done.
    let mut open_count: u128 = 0;
    let closed_count: u128;
    const SEARCH_OPEN: bool = false;
    if SEARCH_OPEN {
        let positions: Vec<u8> = (0..1 as u8).collect();
        let sums: Vec<(u128, u128)> = positions
            .par_iter()
            .map(|start| {
                let mut visited: Visited = [false; BOARD_SIZE];
                visited[*start as usize] = true;
                let stack: Stack = [(*start, 0); BOARD_SIZE];
                let stack_ptr = 0;
                let mut sd = SearchData {
                    visited,
                    stack,
                    stack_ptr,
                    start: *start,
                    open_count: 0,
                    closed_count: 0,
                };
                dfs(&mut sd);
                (sd.open_count, sd.closed_count)
            })
            .collect();
        let (open, closed): (Vec<u128>, Vec<u128>) = sums.into_iter().unzip();
        open_count = open.into_iter().sum();
        closed_count = closed.into_iter().sum();
    } else {
        closed_count = bfs(0, 4);
    }
    println!("Open paths: {}", open_count);
    println!("Directed closed paths: {}", closed_count);
    println!("Undirected closed paths: {}", closed_count / 2);
}

#[derive(Clone, Copy, Debug)]
struct SearchData {
    visited: Visited,
    stack: Stack,
    stack_ptr: usize,
    start: u8,
    open_count: u128,
    closed_count: u128,
}

fn bfs(start: u8, depth_until_dfs: usize) -> u128 {
    let mut visited: Visited = [false; BOARD_SIZE];
    visited[start as usize] = true;
    let stack: Stack = [(start, 0); BOARD_SIZE];
    let stack_ptr = 0;
    let start_sd = SearchData {
        visited,
        stack,
        stack_ptr,
        start,
        open_count: 0,
        closed_count: 0,
    };

    let mut current: Vec<SearchData> = vec![start_sd];
    let mut depth = 0;

    while depth < depth_until_dfs {
        let mut new_current: Vec<SearchData> = vec![];
        // Visit all currents
        for sd in &mut current {
            let (pos, _) = sd.stack[sd.stack_ptr];

            for jump_pos in POSSIBLE_JUMPS[pos as usize] {
                if jump_pos != u8::MAX && sd.visited[jump_pos as usize] == false {
                    let mut sd = sd.clone();
                    sd.visited[jump_pos as usize] = true;
                    sd.stack_ptr += 1;
                    sd.stack[sd.stack_ptr] = (jump_pos, 0);
                    new_current.push(sd);
                }
            }
        }
        depth += 1;
        current = new_current;
    }

    println!("Iterating over {} sub-boards concurrently", current.len());

    let closed_count: u128 = current
        .par_iter_mut()
        .map(|sd| {
            dfs(sd);
            sd.closed_count
        })
        .sum();

    closed_count
}

fn dfs(sd: &mut SearchData) {
    let start = sd.start;

    // If we reach this and want to unwind, break and return.
    let starting_stack_ptr = sd.stack_ptr;

    // let mut visited: Visited = [false; BOARD_SIZE];
    // visited[start as usize] = true;
    // let mut stack: Stack = [(start, 0); BOARD_SIZE];

    loop {
        let (pos, jump_index) = sd.stack[sd.stack_ptr];

        // println!("ROUND");
        // println!("current = ({:?}, {:?})", pos, jump_index);
        // println!("stack_ptr = {}", stack_ptr);

        if jump_index > 7 {
            // let amount: u64 = sd.visited.iter().map(|v| if *v { 1 } else { 0 }).sum();
            // println!("AMOUNT: {}, MAX: {}", amount, BOARD_SIZE);
            // There are no places to jump left. Check if we are done and unwind.
            let can_jump_start = POSSIBLE_JUMPS[pos as usize].iter().any(|j| *j == sd.start);
            let all_visited = sd.visited.iter().all(|v| *v);

            // if can_jump_start {
            //     println!("CAN_JUMP_START");
            //     print_board(&pos, &visited);
            // }
            // if all_visited {
            //     println!("ALL_VISITED");
            //     print_board(&pos, &visited);
            // }

            if all_visited {
                sd.open_count += 1;
            }
            if all_visited && can_jump_start {
                sd.closed_count += 1;
            }

            // Unwind.
            if sd.stack_ptr == starting_stack_ptr {
                break;
            }

            sd.visited[pos as usize] = false;
            sd.stack_ptr -= 1;
            // println!("UNWIND TO: {:?}", stack[stack_ptr].0);
            continue;
        }

        // increase current.1 (jump index);
        sd.stack[sd.stack_ptr].1 = jump_index + 1;

        let jump_pos = POSSIBLE_JUMPS[pos as usize][jump_index as usize];
        // println!("possible = {:?}", jump_pos);

        if jump_pos != u8::MAX && sd.visited[jump_pos as usize] == false {
            // println!("JUMP TO: {:?}", jump_pos);
            // The jump location is valid, so go there.
            sd.visited[jump_pos as usize] = true;
            sd.stack_ptr += 1;
            sd.stack[sd.stack_ptr] = (jump_pos, 0);
        }
    }

    println!(
        "For start {}\t found {}\t open paths and {}\t closed paths.",
        start, sd.open_count, sd.closed_count
    );
}

fn print_board(pos: &u8, visited: &Visited) {
    println!("BOARD");
    for i in 0..BOARD_WIDTH {
        for k in 0..BOARD_WIDTH {
            let index = i * BOARD_WIDTH + k;
            if index == *pos {
                print!("C ");
                continue;
            }
            fn c(v: bool) -> &'static str {
                if v {
                    "T"
                } else {
                    "F"
                }
            }
            print!("{} ", c(visited[index as usize]));
        }
        println!("");
    }
}
