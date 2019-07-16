extern crate diceval;

use std::io;
use std::io::Write;

fn main() {
    let mut buffer = String::new();
    loop {
        buffer.clear();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut buffer).unwrap();
        let result = diceval::parser::parse(buffer.clone()).unwrap();
        println!("Return: {:?}", result);

        for entity in result.iter() {
            println!("Entity: {}", entity.show());
        }
    }
}
