extern crate diceval;

use std::io;

fn main() {
    let mut buffer = String::new();
    loop {
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();
        let result = diceval::parser::parse(buffer.clone());
        println!("Parser result: {:?}", result)
    }
}
