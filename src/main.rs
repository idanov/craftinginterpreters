use std::cell::RefCell;
use std::env;
use std::fs;
use std::process::exit;
use std::rc::Rc;

mod environment;
mod expr;
mod interpreter;
mod lox_callable;
mod parser;
mod resolver;
mod scanner;
mod stmt;

use interpreter::Interpreter;
use parser::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use stmt::Stmt;

use colored::Colorize;

use crate::resolver::Resolver;

struct Lox {
    interpreter: Rc<RefCell<Interpreter>>,
}

impl Lox {
    pub fn new() -> Self {
        Lox {
            interpreter: Rc::new(RefCell::new(Interpreter::new())),
        }
    }

    pub fn run_file(&mut self, filename: &String) {
        let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
        self.run(contents);
        // if had_error {
        //     exit(65);
        // }
        // if (hadRuntimeError) System.exit(70);
    }

    pub fn run_prompt(&mut self) {
        let mut rl = DefaultEditor::new().expect("Something went wrong with starting rustyline");
        loop {
            let readline = rl.readline(">>> ");
            match readline {
                Ok(line) => {
                    let _ = rl.add_history_entry(line.as_str());
                    self.run(line);
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    break;
                }
                Err(err) => {
                    eprintln!("{}", format!("Error: {:?}", err).red());
                    break;
                }
            }
            // had_error = false;
        }
    }

    pub fn run(&mut self, source: String) {
        // scan tokens and print them
        let mut scan = scanner::Scanner::new(&source);
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
        let parsed: Result<Vec<Stmt>, String> = parser.parse();

        if let Err(e) = &parsed {
            eprintln!("{}", e.red());
        }

        let statements: Vec<Stmt> = parsed.unwrap_or_default();
        for x in &statements {
            println!("{}", x);
        }

        println!("-------- Resolver results ------");
        let mut resolver = Resolver::new(self.interpreter.clone());
        if let Err(e) = resolver.resolve(&statements) {
            eprintln!("{}", e.red());
        };

        println!("-------- Interpreter results ------");
        if let Err(e) = self.interpreter.borrow_mut().interpret(&statements) {
            eprintln!("{}", e.red());
        };
    }

    // fn error(line: usize, message: String) {
    //     report(line, "".to_string(), message);
    // }

    // fn report(line: usize, where_str: String, message: String) {
    //     eprintln!("[line {}] Error{}: {}", line, where_str, message);
    // }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut lox = Lox::new();
    match args.len() {
        1 => lox.run_prompt(),
        2 => lox.run_file(&args[1]),
        _ => {
            println!("Usage: craft [script]");
            exit(64);
        }
    }
}
