use std::iter::Extend;
use std::fmt::Debug;

use combine::error::ParseError;
use combine::error::StreamError;
use combine::parser::char::{space, string};
use combine::stream::state::State;
use combine::stream::StreamErrorFor;
use combine::{choice, many, many1, optional, satisfy, skip_many, attempt, Parser, Stream, eof};
use failure::Error;
use super::types::{Dice, Expr, Num, Operator, Entity};



pub fn number<I>() -> impl Parser<Input = I, Output = Num>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::char::digit;
    use combine::parser::repeat::count_min_max;

    let parse_number = |s: String| s.parse::<Num>()
        .map_err(|_| StreamErrorFor::<I>::unexpected_message("fail to parse number"));

    count_min_max(1, 6, digit())
        .and_then(parse_number)
        .expected("number")
}

pub fn dice<I>() -> impl Parser<Input = I, Output = Dice>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::{one_of, not_followed_by};
    use combine::parser::char::letter;
    let d = one_of("Dd".chars())
        .expected("the 'd' or 'D' in the XdY");
    let counter = optional(number());
    let face = optional(number());
    counter.skip(d).skip(not_followed_by(letter())).and(face)
        .map(|(counter, face)| Dice { face, number: counter.unwrap_or(1) })
}

parser!{
    fn expr[I]()(I) -> Expr
    where [I: Stream<Item = char>]
    {
        expr_()
    }
}

pub fn expr_<I>() -> impl Parser<Input = I, Output = Expr>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::{optional, between};
    use combine::parser::char::{char, string};
    let add = char('+').map(|_| Operator::Add);
    let sub = char('-').map(|_| Operator::Sub);
    let mul = char('*').map(|_| Operator::Mul);
    let div = char('/').map(|_| Operator::Div);
    let max = string("max").map(|_| Operator::Max);
    let min = string("min").map(|_| Operator::Min);
    
    let infix = choice((add, sub, mul, div));
    let prefix = choice((attempt(max), min));

    let sub_expr = || choice((
        between(char('('), char(')'), expr().skip(skip_many(space()))),
        between(char('（'), char('）'), expr().skip(skip_many(space()))),
    ));

    let left_parser = choice((
        attempt(dice().map(Expr::Roll)),
        number().map(Expr::Num),
        (prefix, sub_expr()).map(|(op, expr)| Expr::Prefix(op, Box::new(expr))),
        sub_expr(),
    ));
    let rest = attempt(skip_many(space()).with(infix.and(expr())));

    skip_many(space())
        .with(left_parser)
        .and(optional(rest))
        .map(|(left, rest): (Expr, Option<(Operator, Expr)>)| match rest {
            Some((op, right)) => Expr::Infix(Box::new(left), op, Box::new(right)),
            None => left,
        })
}

pub fn entity<I>() -> impl Parser<Input = I, Output = Entity>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::skip_many1;
    use combine::parser::combinator::{recognize};
    use combine::parser::item::{any, satisfy};
    use combine::parser::char::digit;

    let expression = expr().map(Entity::Expr);
    let alphabetic = || satisfy(|c: char| c.is_alphabetic()).expected("tail character");
    let whatever = any().map(|_| ());

    let description = {
        let number = skip_many1(digit());
        let word = skip_many1(alphabetic());
        let body = choice((word, number, whatever));
        let tail = skip_many(space());
        let pattern = (body, tail);
        recognize(pattern).map(Entity::Description)
    };
    choice((attempt(expression), description))
}


pub fn entities<I>() -> impl Parser<Input = I, Output = Vec<Entity>>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    many(entity())
}

pub fn parse<T: AsRef<str>>(s: T) -> impl Debug {
    use combine::stream::IteratorStream;
    entities()
        .easy_parse(State::new(IteratorStream::new(s.as_ref().chars())))
        .map(|(x, _)| x)
}
