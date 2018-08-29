use combine;
use combine::error::ParseError;
use combine::{skip_many, many, many1, optional, Parser, Stream, choice, try, satisfy};
use combine::parser::char::{space, string};
use combine::error::StreamError;
use combine::stream::StreamErrorFor;
use combine::stream::state::State;
use failure::Error;
use types::{Num, Dice, Operator, Expr, Command};

macro_rules! or {
    ($($s:expr),+) => {$crate::combine::choice(($(try(string($s)),)+))};
}


macro_rules! parser {
    ($($name: ident :: $Output: ty $block:block);+;) => {
        $(
            pub fn $name<I>() -> impl Parser<Input = I, Output = $Output>
                where
                    I: Stream<Item = char>,
                    I::Error: ParseError<char, I::Range, I::Position>,
            {
                $block
            }
        )+
    };
}


pub fn number<I>() -> impl Parser<Input = I, Output = Num>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::char::digit;
    many1(digit())
        .and_then(|s: String| s.parse::<Num>()
            .map_err(|_| StreamErrorFor::<I>::unexpected_message("fail to parse number")))
        .expected("number")
}



pub fn dice<I>() -> impl Parser<Input = I, Output = Dice>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<char, I::Range, I::Position>,
{
    let d = or!["D", "d"].expected("the 'd' or 'D' in the XdY");
    let face = {
        let num = number().map(|n| Some(n));
        let none = combine::skip_many1(space()).map(|_| None);
        let eof = combine::eof().map(|_| None);
        choice((num, none, eof))
    };
    optional(number())
        .skip(d)
        .and(face)
        .map(|(n, face)| Dice { face, number: n.unwrap_or(1) })
}

pub fn variable<I>() -> impl Parser<Input = I, Output = String>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::char::{char, letter, digit};
    let ident = choice((letter(), digit(), char('_'), char(':')));
    char('.').with(many1(ident))
}

pub fn expr<I>() -> impl Parser<Input = I, Output = Expr>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<char, I::Range, I::Position>,
{
    choice((
        try(dice()).map(Expr::Dice),
        try(operator()).map(Expr::Operator),
        try(variable()).map(Expr::Variable),
        try(number()).map(Expr::Num),
        description().map(Expr::Description)
    ))
    .expected("an expression such as 1d100")
}


pub fn description<I>() -> impl Parser<Input = I, Output = String>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<char, I::Range, I::Position>,
{
    let some_char = satisfy(|c: char| !c.is_whitespace() && !c.is_control());
    many1(some_char)
}


pub fn roll<I>() -> impl Parser<Input = I, Output = Command>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<char, I::Range, I::Position>,
{
    many(expr().skip(skip_space())).map(|mut xs: Vec<_>| {
        if xs.len() == 0 {
            xs.push(Expr::Dice(Dice::default()));
        }
        Command::Roll(xs)
    })
}

parser!{
    add :: Operator {
        or!["+", "add", "ADD", "加"]
            .map(|_| Operator::Add)
    };

    sub :: Operator {
        or!["-", "sub", "减", "SUB"]
            .map(|_| Operator::Sub)
    };


    mul :: Operator {
        or!["*", "×", "x", "乘", "MUL", "mul"]
            .map(|_| Operator::Mul)
    };

    div :: Operator {
        or!["/", "÷", "除", "DIV", "div"]
            .map(|_| Operator::Div)
    };

    max :: Operator {
        or!["最大", "<=", "max", "MAX"]
            .map(|_| Operator::Max)
    };

    min :: Operator {
        or!["最小", ">=", "min", "MIN"]
            .map(|_| Operator::Min)
    };

    operator :: Operator {
        choice((add(), mul(), sub(), div(), max(), min()))
            .expected("an operator such as +, -, *, /")
    };

    skip_space :: () {
        skip_many(space())
    };
}




pub fn parse<T: AsRef<str>>(s: T) -> Result<Command, Error> {
    use combine::stream::IteratorStream;
    roll()
        .easy_parse(State::new(IteratorStream::new(s.as_ref().chars())))
        .map(|(x, _)| x)
        .map_err(|e| e.into())
}
