const BOARD_WIDTH: u8 = 5;
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
        return u8::MAX;
    }
    if pos < BOARD_WIDTH && jump <= 3 {
        return u8::MAX;
    }
    if pos >= (BOARD_SIZE as u8 - 2 * BOARD_WIDTH) && jump >= 6 {
        return u8::MAX;
    }
    if pos >= (BOARD_SIZE as u8 - BOARD_WIDTH) && jump >= 4 {
        return u8::MAX;
    }
    if pos % BOARD_WIDTH > (BOARD_WIDTH - 3) && (jump == 3 || jump == 5) {
        return u8::MAX;
    }
    if pos % BOARD_WIDTH > (BOARD_WIDTH - 2) && (jump == 1 || jump == 7) {
        return u8::MAX;
    }
    if pos % BOARD_WIDTH < 2 && (jump == 2 || jump == 4) {
        return u8::MAX;
    }
    if pos % BOARD_WIDTH < 1 && (jump == 0 || jump == 6) {
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
        _ => u8::MAX - 1,
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
    println!("{:?}", POSSIBLE_JUMPS);
    let mut sum = 0; // amount of different paths
    let mut visited: Visited = [false; BOARD_SIZE];
    visited[0] = true;
    let mut stack: Stack = [(0, 0); BOARD_SIZE];
    let mut stack_ptr = 0;

    loop {
        let (pos, jump_index) = stack[stack_ptr];

        println!("ROUND");
        for i in 0..BOARD_WIDTH {
            for k in 0..BOARD_WIDTH {
                let index = i * BOARD_WIDTH + k;
                if index == pos {
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
        println!("current = ({:?}, {:?})", pos, jump_index);
        println!("stack_ptr = {}", stack_ptr);

        if jump_index > 7 {
            let amount: u64 = visited.iter().map(|v| if *v { 1 } else { 0 }).sum();
            println!("AMOUNT: {}, MAX: {}", amount, BOARD_SIZE);
            // There are no places to jump left. Check if we are done and unwind.
            if visited.iter().all(|v| *v) {
                sum += 1;
            }

            // Unwind.
            if stack_ptr == 0 {
                break;
            }

            visited[pos as usize] = false;
            stack_ptr -= 1;
            println!("UNWIND TO: {:?}", stack[stack_ptr].0);
            continue;
        }

        // increase current.1 (jump index);
        stack[stack_ptr].1 = jump_index + 1;

        let jump_pos = POSSIBLE_JUMPS[pos as usize][jump_index as usize];
        println!("possible = {:?}", jump_pos);

        if jump_pos != u8::MAX && visited[jump_pos as usize] == false {
            println!("JUMP TO: {:?}", jump_pos);
            // The jump location is valid, so go there.
            visited[jump_pos as usize] = true;
            stack_ptr += 1;
            stack[stack_ptr] = (jump_pos, 0);
        }
    }

    println!("SUM: {:?}", sum);
}
