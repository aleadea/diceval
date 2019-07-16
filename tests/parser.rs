extern crate diceval;
use diceval::types::*;
use diceval::types::Entity::*;
use diceval::types::Expr::*;


fn dice(number: Int, face: Int) -> Dice {
    Dice {
        face: Some(face),
        number,
    }
}

fn roll(dice: Dice) -> Entity {
    Entity::Expression(Expr::Roll(dice))
}

#[test]
fn _2d6() {
    let roll_1d6 = vec![roll(dice(2, 6))];
    let parsed = diceval::parse("2d6".to_string()).unwrap();
    assert_eq!(roll_1d6, parsed);
}

#[test]
fn d() {
    let dice_d = Dice {
        face: None,
        number: 1,
    };
    let roll_d = vec![roll(dice_d)];
    let parsed = diceval::parse("d".to_string()).unwrap();
    assert_eq!(roll_d, parsed);
}

fn desc<T: ToString>(s: T) -> Entity {
    Entity::Description(s.to_string())
}

#[test]
fn pure_description() {
    let s = "晓美焰".to_string();
    let roll_d = vec![desc(s.clone())];
    let parsed = diceval::parse(s).unwrap();
    assert_eq!(roll_d, parsed);
    let s = "鹿目圆香d      d晓美焰";
    let roll_d = vec![desc(s)];
    let parsed = diceval::parse(s.to_string()).unwrap();
    assert_eq!(roll_d, parsed);
}

#[test]
fn description_and_roll1() {
    let dice_d = Dice {
        face: None,
        number: 1,
    };
    let roll_d = vec![desc("小圆roll了 "), roll(dice_d)];
    let parsed = diceval::parse("小圆roll了 d".to_string()).unwrap();
    assert_eq!(roll_d, parsed);

    let dice_d = Dice {
        face: Some(100),
        number: 4,
    };
    let roll_d = vec![desc("小圆roll了 "), roll(dice_d)];
    let parsed = diceval::parse("小圆roll了 4d100".to_string()).unwrap();
    assert_eq!(roll_d, parsed);

    let dice_d = Dice {
        face: Some(100),
        number: 4,
    };
    let roll_d = vec![
        desc("小圆roll了"),
        roll(dice_d),
        desc("小焰"),
    ];
    let parsed = diceval::parse("小圆roll了4d100小焰".to_string()).unwrap();
    assert_eq!(roll_d, parsed);
}

fn add(l: Expr, r: Expr) -> Expr {
    Infix(Box::new(l), Operator::Add, Box::new(r))
}


fn div(l: Expr, r: Expr) -> Expr {
    Infix(Box::new(l), Operator::Div, Box::new(r))
}

fn number(n: Int) -> Expr {
    Num(n)
}

fn max(e: Expr) -> Expr { Prefix(Operator::Max, Box::new(e)) }

fn child(e: Expr) -> Expr { Child(Box::new(e)) }

#[test]
fn expr() {
    let x = number(42);
    let y = number(1);
    let z = number(2);

    let result = vec![Expression(add(x, div(y, z)))];
    let parsed = diceval::parse("42+1/2".to_string()).unwrap();
    assert_eq!(parsed, result);


    let x = number(42);
    let y = number(1);
    let z = number(2);
    let result = vec![Expression(div(child(add(x, y)), z))];
    let parsed = diceval::parse("( 42 + 1 ) / 2".to_string()).unwrap();
    assert_eq!(parsed, result);



    let x = number(42);
    let y = number(1);
    let z = number(2);
    let result = vec![Expression(add(x, add(y, z)))];
    let parsed = diceval::parse("42 + 1 + 2".to_string()).unwrap();
    assert_eq!(parsed, result);

    let result = vec![Description("max max max".to_string())];
    let parsed = diceval::parse("max max max".to_string()).unwrap();
    assert_eq!(parsed, result);

    let max_42 = max(number(42));
    let result = vec![Expression(max(max(max_42)))];
    let parsed = diceval::parse("max max max 42".to_string()).unwrap();
    assert_eq!(parsed, result);
}
