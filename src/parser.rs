use combine::error::ParseError;
use combine::error::StreamError;
use combine::stream::state::State;
use combine::stream::StreamErrorFor;
use combine::{choice, many, optional, attempt, Parser, Stream};
use super::types::{Dice, Expr, Int, Operator, Entity};


pub fn insensitive_string<'a, I>(string: &'static str) -> impl Parser<Input = I, Output = &'a str>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::parser::char::string_cmp;

    string_cmp(string, |l, r| l.eq_ignore_ascii_case(&r))
}


pub fn skip_spaces<I>() -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::skip_many;
    use combine::parser::char::space;

    skip_many(space())
}


pub fn number<I>() -> impl Parser<Input = I, Output =Int>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::char::digit;
    use combine::parser::repeat::count_min_max;

    let parse_number = |s: String| s.parse::<Int>()
        .map_err(|_| StreamErrorFor::<I>::unexpected_message("fail to parse number"));

    count_min_max(1, 6, digit())
        .and_then(parse_number)
        .expected("number")
}

/// Parse dice rolling.
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

/// Infix operators with priority 1.
///
/// such as "+", "-".
pub fn infix_1<I>() -> impl Parser<Input = I, Output = Operator>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::parser::char::{char};
    let add = choice((char('+'), char('加'),)).map(|_| Operator::Add);
    let sub = choice((char('-'), char('减'),)).map(|_| Operator::Sub);
    choice((add, sub))
}

/// Infix operators with priority 2.
///
/// such as "*" "/".
pub fn infix_2<I>() -> impl Parser<Input = I, Output = Operator>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::parser::char::{char};
    let mul = choice((char('*'), char('×'), char('乘'),)).map(|_| Operator::Mul);
    let div = choice((char('/'), char('÷'), char('除'),)).map(|_| Operator::Div);
    choice((mul, div))
}

/// Prefix operators.
pub fn prefix<I>() -> impl Parser<Input = I, Output = Operator>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::parser::char::{string};
    let max = choice((string("最大"), insensitive_string("max"))).map(|_| Operator::Max);
    let min = choice((string("最小"), insensitive_string("min"))).map(|_| Operator::Min);
    choice((attempt(max), min))
}


// recursive parser
parser!{
    fn expr[I]()(I) -> Expr
    where [I: Stream<Item = char>]
    {
        expr_()
    }
}

parser!{
    fn expr_2[I]()(I) -> Expr
    where [I: Stream<Item = char>]
    {
        expr_2_()
    }
}


parser!{
    fn terminal[I]()(I) -> Expr
    where [I: Stream<Item = char>]
    {
        terminal_()
    }
}


fn make_infix_expression((left, rest): (Expr, Option<(Operator, Expr)>)) -> Expr {
    match rest {
        Some((op, right)) => Expr::Infix(Box::new(left), op, Box::new(right)),
        None => left,
    }
}


pub fn terminal_<I>() -> impl Parser<Input = I, Output = Expr>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::between;
    use combine::parser::char::char;

    // (...expr..)
    let child_expr = || choice((
        between(char('('), char(')'), expr().skip(skip_spaces())),
        between(char('（'), char('）'), expr().skip(skip_spaces())),
    ));

    // max(...) min(...)
    let prefix_expr = prefix()
        .skip(skip_spaces())
        .and(terminal())
        .map(|(op, expr)| Expr::Prefix(op, Box::new(expr)));

    // 1d20 | 42 | max(...) | (...)
    choice((
        attempt(dice().map(Expr::Roll)),
        number().map(Expr::Num),
        prefix_expr,
        child_expr(),
    ))
}

/// Parse higher priority expressions.
pub fn expr_2_<I>() -> impl Parser<Input = I, Output = Expr>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    // rest part.
    let rest = skip_spaces().with(infix_2().and(expr_2()));

    skip_spaces()
        .with(terminal())
        .and(optional(attempt(rest)))
        .map(make_infix_expression)
}

/// Parse expressions.
pub fn expr_<I>() -> impl Parser<Input = I, Output = Expr>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    // remove left recursion
    let rest = skip_spaces().with(infix_1().and(expr()));

    skip_spaces()
        // get left expression.
        .with(expr_2())
        // get lowest priority rest part. (operator + - and right)
        .and(optional(attempt(rest)))
        .map(make_infix_expression)
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

    let expression = expr().map(Entity::Expression);

    let description = {
        let alphabetic = || satisfy(|c: char| c.is_alphabetic()).expected("tail character");
        let whatever = any().map(|_| ());
        let number = skip_many1(digit());
        let word = skip_many1(alphabetic());
        let body = choice((word, number, whatever));
        let tail = skip_spaces();
        let pattern = (body, tail);
        // collect consumed tokens.
        recognize(pattern).map(Entity::Description)
    };
    choice((attempt(expression), description))
}


pub fn entities<I>() -> impl Parser<Input = I, Output = Vec<Entity>>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    // concatenate adjacently description entities.
    fn concatenate(mut entities: Vec<Entity>, entity: Entity) -> Vec<Entity> {
        if let Some(last) = entities.last_mut() {
            if let Entity::Description(ref current) = entity {
                if let Entity::Description(prev) = last {
                    prev.push_str(current);
                    return entities;
                }
            }
        }
        entities.push(entity);
        return entities;
    }
    many(entity()).map(|entities: Vec<Entity>| entities.into_iter().fold(Vec::new(), concatenate))
}

pub fn parse<T: AsRef<str>>(s: T) -> Option<Vec<Entity>> {
    use combine::stream::IteratorStream;
    entities()
        .easy_parse(State::new(IteratorStream::new(s.as_ref().chars())))
        .map(|(x, _)| x)
        .ok()
}
