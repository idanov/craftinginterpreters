use std::env;
use std::process::exit;
use std::fs;
use std::io;
use std::io::Write;

mod scanner;

fn main() {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        println!("Usage: dataloch [script]");
        exit(64);
    } else if args.len() == 2 {
        run_file(&args[1])
    } else {
        run_prompt()
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
