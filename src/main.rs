use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process::exit;

mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
mod environment;

use interpreter::Interpreter;
use parser::Parser;
use stmt::Stmt;

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
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    run(contents);
    // if had_error {
    //     exit(65);
    // }
    // if (hadRuntimeError) System.exit(70);
}

fn run_prompt() {
    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        let mut contents = String::new();
        io::stdin()
            .read_line(&mut contents)
            .expect("Something went wrong reading the line");
        run(contents);
        // had_error = false;
    }
}

fn run(source: String) {
    // scan tokens and print them
    let mut scan = scanner::Scanner::new(&source);
    let mut interpreter = Interpreter::new();
    let raw_tokens = scan.scan_tokens();
    println!("-------- Scanner results ------");
    for token in raw_tokens {
        println!("{:?}", token);
    }
    println!("-------- Parser results ------");
    let tokens = raw_tokens
        .into_iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    let mut parser = Parser::new(tokens);
    let statements: Result<Vec<Stmt>, String> = parser.parse();

    match &statements {
        Ok(xs) => {
            for x in xs {
                println!("{}", x);
            }
        }
        Err(e) => eprintln!("{}", e),
    }
    println!("-------- Interpreter results ------");
    if let Err(e) = statements.and_then(|x| interpreter.interpret(&x)) {
        eprintln!("{}", e);
    };
}

// fn error(line: usize, message: String) {
//     report(line, "".to_string(), message);
// }

// fn report(line: usize, where_str: String, message: String) {
//     eprintln!("[line {}] Error{}: {}", line, where_str, message);
// }
