extern crate combine;
#[macro_use]
extern crate failure;
extern crate rand;

mod evaluator;
mod parser;
pub mod types;
use types::Command;

pub fn parse(s: String) -> Command {
    parser::parse(s).unwrap_or(Command::Unsupported)
}

pub fn eval(s: String) -> Result<(types::Num, String), failure::Error> {
    let context = evaluator::Context::new();
    let command = parser::parse(s)?;
    match command {
        types::Command::Roll(e) => {
            let result = evaluator::eval_roll(&context, e)?;
            Ok(result)
        }
        _ => Err(format_err!("unsupported")),
    }
}
