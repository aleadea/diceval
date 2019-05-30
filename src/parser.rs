use combine::error::ParseError;
use combine::error::StreamError;
use combine::stream::state::State;
use combine::stream::StreamErrorFor;
use combine::{choice, many, optional, attempt, Parser, Stream};
use super::types::{Dice, Expr, Num, Operator, Entity};


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


parser!{
    fn expr_1[I]()(I) -> Expr
    where [I: Stream<Item = char>]
    {
        expr_1_()
    }
}

parser!{
    fn expr_2[I]()(I) -> Expr
    where [I: Stream<Item = char>]
    {
        expr_2_()
    }
}

fn infix_mapper((left, rest): (Expr, Option<(Operator, Expr)>)) -> Expr {
    match rest {
        Some((op, right)) => Expr::Infix(Box::new(left), op, Box::new(right)),
        None => left,
    }
}

pub fn expr_2_<I>() -> impl Parser<Input = I, Output = Expr>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::{optional, between};
    use combine::parser::char::char;

    let sub_expr = || choice((
        between(char('('), char(')'), expr_1().skip(skip_spaces())),
        between(char('（'), char('）'), expr_1().skip(skip_spaces())),
    ));

    let prefix_expr = prefix()
        .skip(skip_spaces())
        .and(sub_expr())
        .map(|(op, expr)| Expr::Prefix(op, Box::new(expr)));

    let left_parser = choice((
        attempt(dice().map(Expr::Roll)),
        number().map(Expr::Num),
        prefix_expr,
        sub_expr(),
    ));
    let rest = skip_spaces().with(infix_2().and(expr_2()));

    skip_spaces()
        .with(left_parser)
        .and(optional(attempt(rest)))
        .map(infix_mapper)
}

pub fn expr_1_<I>() -> impl Parser<Input = I, Output = Expr>
where
    I: Stream<Item = char>,
    I::Error: ParseError<char, I::Range, I::Position>,
{
    use combine::optional;

    let rest = skip_spaces().with(infix_1().and(expr_1()));

   skip_spaces()
        .with(expr_2())
        .and(optional(attempt(rest)))
        .map(infix_mapper)
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

    let expression = expr_1().map(Entity::Expr);
    let alphabetic = || satisfy(|c: char| c.is_alphabetic()).expected("tail character");
    let whatever = any().map(|_| ());

    let description = {
        let number = skip_many1(digit());
        let word = skip_many1(alphabetic());
        let body = choice((word, number, whatever));
        let tail = skip_spaces();
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
        .map(|entities: Vec<Entity>| {
            let mut result = Vec::new();
            for entity in entities {
                if let Some(last) = result.last_mut() {
                    if let Entity::Description(ref current) = entity {
                        if let Entity::Description(prev) = last {
                            prev.push_str(current);
                            continue;
                        }
                    }
                }
                result.push(entity);
            }
            return result;
        })
}

pub fn parse<T: AsRef<str>>(s: T) -> Option<Vec<Entity>> {
    use combine::stream::IteratorStream;
    entities()
        .easy_parse(State::new(IteratorStream::new(s.as_ref().chars())))
        .map(|(x, _)| x)
        .ok()
}
