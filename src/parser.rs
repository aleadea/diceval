use combine::error::ParseError;
use combine::error::StreamError;
use combine::parser::char::{space, string};
use combine::stream::state::State;
use combine::stream::StreamErrorFor;
use combine::{choice, many, many1, optional, satisfy, skip_many, attempt, Parser, Stream, eof};
use failure::Error;
use super::types::{Dice, Expr, Num, Operator};

macro_rules! or {
    ($($s:expr),+) => {combine::choice(($(attempt(string($s)),)+))};
}


pub fn number<I>() -> impl Parser<Input = I, Output = Num>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::char::digit;
    many1(digit())
        .and_then(|s: String| {
            s.parse::<Num>()
                .map_err(|_| StreamErrorFor::<I>::unexpected_message("fail to parse number"))
        })
        .expected("number")
}

pub fn dice<I>() -> impl Parser<Input = I, Output = Expr>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    let d = or!["D", "d"].expected("the 'd' or 'D' in the XdY");
    let face = {
        let num = number().map(|n| Some(n));
        let none = space().map(|_| None);
        let eof = eof().map(|_| None);
        choice((num, none, eof))
    };
    optional(number()).skip(d).and(face)
        .map(|(n, face)| Dice { face, number: n.unwrap_or(1) })
        .map(Expr::Dice)
}

pub fn variable<I>() -> impl Parser<Input = I, Output = Expr>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::char::{char, digit, letter};
    let identifier_char = choice((letter(), digit(), char('_'), char(':')));
    let identifier = many1(identifier_char);
    char('\'').with(identifier).map(Expr::Variable)
}

pub fn expr<I>() -> impl Parser<Input = I, Output = Expr>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    choice((
        attempt(dice()),
        attempt(arithmetic()),
        attempt(max_and_min()),
        attempt(variable()),
        attempt(number()).map(Expr::Num),
        description(),
    )).expected("an expression such as 1d100")
}

pub fn description<I>() -> impl Parser<Input = I, Output = Expr>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    let some_char = satisfy(|c: char| !c.is_whitespace() && !c.is_control() && !c.is_numeric() && c != '\'');
    many1(some_char).map(Expr::Description)
}

pub fn roll<I>() -> impl Parser<Input = I, Output = Vec<Expr>>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    let expression = expr().skip(skip_many(space()));
    many(expression)
}


pub fn arithmetic<I>() -> impl Parser<Input = I, Output = Expr>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    let add = or!["+", "add", "ADD", "加"].map(|_| Operator::Add);
    let sub = or!["-", "sub", "减", "SUB"].map(|_| Operator::Sub);
    let mul = or!["*", "×", "x", "乘", "MUL", "mul"].map(|_| Operator::Mul);
    let div = or!["/", "÷", "除", "DIV", "div"].map(|_| Operator::Div);
    let simple_operator = choice((add, sub, mul, div));
    let oprand = choice((attempt(dice()), attempt(variable()), attempt(max_and_min()), number().map(Expr::Num)));
    (simple_operator, skip_many(space()), oprand)
        .map(|(operator, _, expr)| Expr::Operation(operator, Box::new(expr)))
}

pub fn max_and_min<I>() -> impl Parser<Input = I, Output = Expr>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    let max = or!["最大", "max", "MAX"].map(|_| Operator::Max);
    let min = or!["最小", "min", "MIN"].map(|_| Operator::Min);
    let list_operator = choice((max, min));
    (list_operator, skip_many(space()), dice())
        .map(|(operator, _, expr)| Expr::Operation(operator, Box::new(expr)))
}

pub fn parse<T: AsRef<str>>(s: T) -> Result<Vec<Expr>, Error> {
    use combine::stream::IteratorStream;
    roll()
        .easy_parse(State::new(IteratorStream::new(s.as_ref().chars())))
        .map(|(x, _)| x)
        .map_err(|e| e.into())
}
