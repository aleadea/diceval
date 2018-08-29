extern crate diceval;
use diceval::types::*;
use diceval::types::Command::Roll;

#[test]
fn _1d6() {
    let dice_1d6 = Dice {face: Some(6), number: 1};
    let roll_1d6 = Roll(vec![Expr::Dice(dice_1d6)]);
    let parsed = diceval::parse("1d6".to_string());
    assert_eq!(roll_1d6, parsed);
}


#[test]
fn d() {
    let dice_d = Dice {face: None, number: 1};
    let roll_d = Roll(vec![Expr::Dice(dice_d)]);
    let parsed = diceval::parse("d".to_string());
    assert_eq!(roll_d, parsed);
}


fn desc<T: ToString>(s: T) -> Expr {
    Expr::Description(s.to_string())
}


#[test]
fn pure_description1() {
    // let dice_d = Dice {face: None, number: 1};
    let s = "晓美焰".to_string();
    let roll_d = Roll(vec![desc(s.clone())]);
    let parsed = diceval::parse(s);
    assert_eq!(roll_d, parsed);
}

#[test]
fn pure_description2() {
    // let dice_d = Dice {face: None, number: 1};
    let roll_d = Roll(vec![desc("鹿目圆香d"), desc("d晓美焰")]);
    let parsed = diceval::parse("鹿目圆香d      d晓美焰".to_string());
    assert_eq!(roll_d, parsed);
}


#[test]
fn description_and_roll1() {
    let dice_d = Dice {face: None, number: 1};
    let roll_d = Roll(vec![desc("小圆roll了"), Expr::Dice(dice_d)]);
    let parsed = diceval::parse("小圆roll了 d".to_string());
    assert_eq!(roll_d, parsed);
}


#[test]
fn description_and_roll2() {
    let dice_d = Dice {face: Some(100), number: 4};
    let roll_d = Roll(vec![desc("小圆roll了"), Expr::Dice(dice_d)]);
    let parsed = diceval::parse("小圆roll了 4d100".to_string());
    assert_eq!(roll_d, parsed);
}


#[test]
fn description_and_roll3() {
    let dice_d = Dice {face: Some(100), number: 4};
    let roll_d = Roll(vec![desc("小圆roll了"), Expr::Dice(dice_d), desc("小焰除3")]);
    let parsed = diceval::parse("小圆roll了 4d100小焰除3".to_string());
    assert_eq!(roll_d, parsed);
}


#[test]
fn number() {
    let n = Expr::Num(42);
    let cmd = Roll(vec![n]);
    let parsed = diceval::parse("42".to_string());
    assert_eq!(parsed, cmd);
}
