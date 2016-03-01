A Brainfuck interpreter in Rust. `src/main.rs` is only a driver. For the 
interesting code, see `src/brainfuck/mod.rs`.

The instruction tape is represented by a simple vector of enums representing 
instructions. I chose a vector because the code is expected to be relatively 
dense (i.e. little wasted space between used cells).

The data tape is represented by a hash map of i64's. I chose hashmap because 
(I don't think) I can make good assumptions about the density of the data. The
hashmap offers a good compromise resulting in good performance for dense and
sparse usage of the data tape.
