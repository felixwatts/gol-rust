extern crate regex;
extern crate num;
extern crate termion;

use std::io;
use std::io::{Write, stdout};
use std::collections::HashSet;
use regex::Regex;
use termion::raw::IntoRawMode;

// State of the board
struct Board {
    width: u64,
    height: u64,
    living_cells: HashSet<(u64, u64)> 
}

impl Board {

    // true if the cell is within the bounds of the board
    fn contains(&self, cell: &(u64, u64)) -> bool {
        cell.0 < self.width && cell.1 < self.height
    }

    // returns the neighbors of the specified cell
    fn neighbors_of(&self, cell: &(u64, u64)) -> HashSet<(u64, u64)> {

        let x = cell.0;
        let y = cell.1;

        let x_lo = if x == 0 { 0 } else { x - 1 };
        let y_lo = if y == 0 { 0 } else { y - 1 };

        let x_hi = if x == u64::max_value() { u64::max_value() } else { x + 1 };
        let y_hi = if y == u64::max_value() { u64::max_value() } else { y + 1 };

        let mut n: HashSet<(u64, u64)> = [
            (x_lo, y_lo),
            (x_lo, y),
            (x_lo, y_hi),
            (x, y_lo),
            (x, y_hi),
            (x_hi, y_lo),
            (x_hi, y),
            (x_hi, y_hi)
            ]        
            .iter()
            .cloned()
            .collect();

        n.retain(|x| { self.contains(x) });        

        n
    }

    // returns the number of living neighbors of the scpecified cell
    fn count_living_neighbors_of(&self, cell: &(u64, u64)) -> usize {
        self
            .neighbors_of(cell)
            .intersection(&self.living_cells)
            .count()
    }

    // true if the specified cell is alive
    fn is_alive(&self, cell: &(u64, u64)) -> bool {
        self.living_cells.contains(cell)
    }

    // kill the specified cell. NOP if the cell is not alive
    fn kill(&mut self, cell: &(u64, u64)) {
        self.living_cells.remove(cell);
    }

    // set the specified cell to alive. NOP is the cell is already alive
    fn spawn(&mut self, cell: &(u64, u64)) {
        self.living_cells.insert(cell.clone());
    }

    // update the state of the specfied cell once
    fn update_cell(&mut self, cell: &(u64, u64)) {

        match self.contains(cell) {
            true => {
                let is_alive = self.is_alive(cell);
                let num_living_neighbors = self.count_living_neighbors_of(cell);
                match (is_alive, num_living_neighbors) {
                    (true, 2) => return,
                    (true, 3) => return,
                    (true, _) => self.kill(cell),
                    (false, 3) => self.spawn(cell),
                    (false, _) => return
                }
            },
            false => return
        }
    }

    // returns all the cells that can possibly change state in the next update
    // which is all the living cells and their neighbors
    fn cells_that_need_update(&self) -> HashSet<(u64, u64)> {
        let mut r = HashSet::new();
        for living_cell in self.living_cells.iter() {
            for neighbor in self.neighbors_of(&living_cell).iter() {
                r.insert(neighbor.clone());
            }
            r.insert(living_cell.clone());
        }
        r      
    }

    // update the whole board once
    fn update(&mut self) {
        for cell in self.cells_that_need_update() {
            self.update_cell(&cell);
        }
    }

    // print a '🐞' to the console at the specirfied position with the specified offset
    fn print_cell(&self, stdout: &mut Write, x_off: &u64, y_off: &u64, cell: &(u64, u64)) {
        let x_screen = cell.0.clone() - x_off;
        let y_screen = cell.1.clone() - y_off;

        write!(stdout, "{}🐞", termion::cursor::Goto(x_screen as u16+1, y_screen as u16 +1)).expect("Failed to write to console");
    }

    // print a graphical representation of the board state to the console
    fn print(&self) {   

        match self.living_cells.is_empty() {
            true => {
                println!("There are no living cells");                
            },
            false => {
                let x_off = &self.living_cells.iter().min_by_key(|&c| { c.0.clone() }).unwrap().0;
                let y_off = &self.living_cells.iter().min_by_key(|&c| { c.1.clone() }).unwrap().1;

                let mut stdout = stdout().into_raw_mode().unwrap();

                write!(stdout, "{}", termion::clear::All).expect("Failed to write to console");                

                for cell in &self.living_cells {
                    self.print_cell(&mut stdout, &x_off, &y_off, &cell);
                }

                let y_max = &self.living_cells.iter().max_by_key(|&c| { c.1.clone() }).unwrap().1 - y_off;
                writeln!(stdout, "{}", termion::cursor::Goto(1, y_max as u16 /*.to_u16().unwrap()*/+1)).expect("Failed to write to console");
            }
        }        
    }
}

// parse the specfied capture to the specified type
fn capture_value<T>(captures: &regex::Captures, index: usize) -> T 
    where T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug {
    let x : T = captures
            .get(index)
            .unwrap()
            .as_str()
            .parse()
            .unwrap();
    x
}

// handle the CLI 'get' command: 'get x y'
fn handle_get(board: &mut Board, captures: &regex::Captures) {
    let x : u64 = capture_value(&captures, 1);
    let y : u64 = capture_value(&captures, 2);

    let is_alive = board.is_alive(&(x, y));

    println!("{}", is_alive);
}

// handle the CLI 'set' command: 'set x y v'
fn handle_set(board: &mut Board, captures: &regex::Captures) {
    let x : u64 = capture_value(&captures, 1);
    let y : u64 = capture_value(&captures, 2);
    let v : bool = capture_value(&captures, 3);

    match v {
        true => board.spawn(&(x, y)),
        false => board.kill(&(x, y))
    }
}

// handle the CLI 'next' command: 'next'
fn handle_next(board: &mut Board, _: &regex::Captures) {
    board.update();
}

// handle the CLI 'run' command: 'run n'
fn handle_run(board: &mut Board, captures: &regex::Captures) {
    let n : u32 = capture_value(&captures, 1);
    for _ in 0..n {
        board.update();
    }
}

// handle the CLI 'anim' command: 'anim n'
fn handle_anim(board: &mut Board, captures: &regex::Captures) {
    let n : u32 = capture_value(&captures, 1);
    for _ in 0..n {
        board.update();
        board.print();
    }
}

// handle the CLI 'print' command: 'print'
fn handle_print(board: &mut Board, _: &regex::Captures) {
    board.print();
}

// handle the CLI default command: '.*'
fn handle_default(_: &mut Board, captures: &regex::Captures) {
    let cmd : String = capture_value(&captures, 0);
    println!("Unknown command: {}", cmd);
    println!("Use one of the following:");
    println!("");
    println!("set x:int y:int v:bool   set the state of a cell e.g.: set 5 9 true");
    println!("get x:int y:int          get the state of a cell e.g.: get 5 9");
    println!("next                     advance the state by one step");
    println!("run n:int                advance the state by n steps e.g.: run 10");
    println!("anim n:int               advance the state by n steps and print the board after each step e.g.: anim 10");
    println!("print                    display the current state");
    println!("exit                     exit the application");
}

// a type representing a function for handling a CLI command
type CommandHandler = fn(&mut Board, &regex::Captures);
// a type representing a CLI command regex string
type Command = &'static str;

// a mapping of command regex string to command handler function
const CMDS : &'static [(Command, CommandHandler)] = &[
    (r"^get ([0-9]+) ([0-9]+)$", handle_get),
    (r"^set ([0-9]+) ([0-9]+) (true|false)$", handle_set),
    (r"^next$", handle_next),
    (r"^run ([0-9]+)$", handle_run),
    (r"^anim ([0-9]+)$", handle_anim),
    (r"^print$", handle_print),
    (r".*", handle_default)];

// enum representing what to do after processing a command
enum ContinueOrExit {
    Continue,
    Exit
}

// process the specified CLI input string
fn process_input(input: &str, board: &mut Board) -> ContinueOrExit {

    if input == "exit" {
        return ContinueOrExit::Exit
    }

    for cmd in CMDS {    

        let regex = Regex::new(cmd.0).unwrap();

        if regex.is_match(&input) {
            let captures = regex.captures(&input).unwrap();

            cmd.1(board, &captures);
            return ContinueOrExit::Continue
        }
    }

    assert!(false);
    ContinueOrExit::Exit
}

// read a CLI input string from stdin
fn read_input(buffer: &mut String) -> &str {
    print!("> ");
    io::stdout().flush().unwrap();

    io::stdin().read_line(buffer)
        .expect("Failed to read line");
    return buffer.trim()
}

// read and process a CLI input
fn read_and_process_input(board: &mut Board) -> ContinueOrExit {
    let mut input = String::new();
    let input = read_input(&mut input);
    process_input(&input, board)
}

fn main() {

    let mut board = Board { 
        width: 30,
        height: 30,
        living_cells: HashSet::new()
    };    

    loop {
        match read_and_process_input(&mut board) {
            ContinueOrExit::Continue => { continue } 
            ContinueOrExit::Exit => { break } 
        };
    }    

    println!("Bye!");
}
