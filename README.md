# Crafting Interpreters

This project is my own attempt to learn Rust by doing. I went through the book https://craftinginterpreters.com/
converting the Java examples to Rust without knowing much Rust, which might be evident by some design choices I made.
Towards the end of the first part of the book I started looking for ways to improve the code a bit and I also added the
original tests and tweaked them a bit to work with my code. The tests were really helpful in finding a few subtle bugs
I had missed initially.

# How to run the tests

```sh
cargo test
```

# How to run the interpreter with a specific program

```sh
cargo run -- test/_my/programs/non-trivial.lox
```

# Benchmark tests

All benchmark tests are run with `cargo run`, which means they are unoptimized and with debuginfo symbols embedded.

| Benchmark | Time (s) |
|-----------|-------------|
| [binary_trees.lox](test/benchmark/binary_trees.lox) | 561.40 |
| [equality.lox](test/benchmark/equality.lox) | 83.84; 87.87; 4.02 |
| [fib.lox](test/benchmark/fib.lox) | 283.67 |
| [instantiation.lox](test/benchmark/instantiation.lox) | 85.20 |
| [invocation.lox](test/benchmark/invocation.lox) | 65.49 |
| [method_call.lox](test/benchmark/method_call.lox) | 69.07 |
| [properties.lox](test/benchmark/properties.lox) | 134.05 |
| [string_equality.lox](test/benchmark/string_equality.lox) | 170.44; 172.62; 2.17 |
| [trees.lox](test/benchmark/trees.lox) | 1281.16 |
| [zoo.lox](test/benchmark/zoo.lox) | 205.78 |
| [zoo_batch.lox](test/benchmark/zoo_batch.lox) | 10.20 |
