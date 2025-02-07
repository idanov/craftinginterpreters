use log::debug;
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

    pub fn run_file(&mut self, filename: &str) -> i32 {
        let contents =
            fs::read_to_string(filename).expect("Something went wrong reading the file...");
        match self.run(&contents) {
            Ok(()) => 0,
            Err(err) => err,
        }
    }

    pub fn run_prompt(&mut self) {
        let mut rl = DefaultEditor::new().expect("Something went wrong with starting rustyline...");
        loop {
            let readline = rl.readline(">>> ");
            match readline {
                Ok(line) => {
                    let _ = rl.add_history_entry(line.as_str());
                    let _ = self.run_repl(&line);
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
        }
    }

    pub fn run_repl(&mut self, source: &str) -> Result<(), i32> {
        // scan tokens and print them
        let mut scan = scanner::Scanner::new(source);
        let raw_tokens = scan.scan_tokens();
        debug!("-------- Scanner results ------");
        for token in raw_tokens {
            debug!("{:?}", token);
            if let Err(e) = token {
                eprintln!("{}", e.red());
            }
        }
        debug!("-------- Parser results (expr) ------");
        let tokens = raw_tokens.iter().flatten().cloned().collect::<Vec<_>>();
        let mut parser = Parser::new(tokens);
        if let Ok(expr) = parser.parse_expr() {
            let res = self.interpreter.borrow_mut().evaluate(&expr);
            return match res {
                Ok(val) => {
                    println!("{}", val);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("{}", e.red());
                    Err(70)
                }
            };
        }
        Err(65)
    }

    pub fn run(&mut self, source: &str) -> Result<(), i32> {
        // scan tokens and print them
        let mut scan = scanner::Scanner::new(source);
        let raw_tokens = scan.scan_tokens();
        debug!("-------- Scanner results ------");
        for token in raw_tokens {
            debug!("{:?}", token);
            if let Err(e) = token {
                eprintln!("{}", e.red());
                return Err(65);
            }
        }
        debug!("-------- Parser results (stmt) ------");
        let tokens = raw_tokens.iter().flatten().cloned().collect::<Vec<_>>();
        let mut parser = Parser::new(tokens);
        let parsed: Result<Vec<Stmt>, String> = parser.parse();

        if let Err(e) = &parsed {
            eprintln!("{}", e.red());
            return Err(65);
        }

        let statements: Vec<Stmt> = parsed.unwrap_or_default();
        for x in &statements {
            debug!("{}", x);
        }

        debug!("-------- Resolver results ------");
        let mut resolver = Resolver::new(self.interpreter.clone());
        if let Err(e) = resolver.resolve(&statements) {
            eprintln!("{}", e.red());
            return Err(65);
        }
        debug!("-------- Interpreter results ------");
        if let Err(e) = self.interpreter.borrow_mut().interpret(&statements) {
            eprintln!("{}", e.red());
            return Err(70);
        };
        Ok(())
    }
}

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let mut lox = Lox::new();
    match args.len() {
        1 => lox.run_prompt(),
        2 => exit(lox.run_file(&args[1])),
        _ => {
            println!("Usage: rjlox [script]");
            exit(64);
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use regex::Regex;
    use rstest::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn expected(path: &Path) -> String {
        let contents = fs::read_to_string(path).unwrap();

        let expected: String = contents
            .lines()
            .filter(|line| line.contains("// expect: "))
            .map(|line| line.split("// expect: ").nth(1).unwrap().to_string() + "\n")
            .collect();
        expected
    }
    fn expected_runtime_error(path: &Path) -> String {
        let contents = fs::read_to_string(path).unwrap();

        let expected: String = contents
            .lines()
            .filter(|line| line.contains("// expect runtime error: "))
            .map(|line| {
                line.split("// expect runtime error: ")
                    .nth(1)
                    .unwrap()
                    .to_string()
                    + "\n"
            })
            .collect();
        expected
    }

    fn expected_error_at(path: &Path) -> String {
        let contents = fs::read_to_string(path).unwrap();
        let re = Regex::new(r"//.* Error.*").unwrap();

        let expected: String = contents
            .lines()
            .filter(|line| re.is_match(line))
            .map(|line| line.split("// ").nth(1).unwrap().to_string() + "\n")
            .collect();
        expected
    }

    #[rstest]
    #[trace]
    fn test_interpreter(
        #[files("test/**/*.lox")]
        #[exclude("test/benchmark")] // this is benchmark tests
        #[exclude("test/expressions")] // this is for the expressions eval
        #[exclude("test/scanning")] // this is just for the scanner
        #[exclude("test/limit")] // this is for the compiler
        path: PathBuf,
    ) {
        let mut cmd = Command::cargo_bin("rjlox").unwrap();
        let successful = expected(&path);
        let runtime_error = expected_runtime_error(&path);
        let error = expected_error_at(&path);
        if runtime_error.len() > 0 {
            cmd.arg(&path)
                .assert()
                .failure()
                .code(70)
                .stderr(runtime_error);
        } else if error.len() > 0 {
            cmd.arg(&path).assert().failure().code(65).stderr(error);
        } else {
            cmd.arg(&path).assert().success().stdout(successful);
        }
    }
}
