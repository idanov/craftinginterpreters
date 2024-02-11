use std::env;
use std::process::exit;
use std::fs;
use std::io;
use std::io::Write;

mod scanner;
mod parser;
mod expr;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => run_prompt(),
        2 => run_file(&args[1]),
        _ => {
            println!("Usage: craft [script]");
            exit(64);
        }
    }
}

fn run_file(filename: &String) {
    let contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");
    run(contents);
    // if had_error {
    //     exit(65);
    // }
}

fn run_prompt() {
    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        let mut contents = String::new();
        io::stdin().read_line(&mut contents)
            .expect("Something went wrong reading the line");
        run(contents);
        // had_error = false;
    }
}

fn run(source: String) {
    // scan tokens and print them
    let mut scan = scanner::Scanner::new(&source);
    let tokens = scan.scan_tokens();
    for token in tokens {
        println!("{:?}", token);
    }
}

// fn error(line: usize, message: String) {
//     report(line, "".to_string(), message);
// }

// fn report(line: usize, where_str: String, message: String) {
//     eprintln!("[line {}] Error{}: {}", line, where_str, message);
// }
