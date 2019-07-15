pub type Int = i64;

#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Hash, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Max,
    Min,
}

impl Operator {
    pub fn show(&self) -> &'static str {
        use self::Operator::*;
        match self.clone() {
            Add => "+",
            Sub => "-",
            Mul => "×",
            Div => "÷",
            Max => "max",
            Min => "min",
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Hash, Clone, Copy)]
pub struct Dice {
    pub face: Option<Int>,
    pub number: Int,
}

impl Default for Dice {
    fn default() -> Dice {
        Dice {
            face: None,
            number: 1,
        }
    }
}


#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Hash, Clone)]
pub enum Expr {
    Prefix(Operator, Box<Expr>),
    Infix(Box<Expr>, Operator, Box<Expr>),
    Num(Int),
    Roll(Dice),
}


#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Hash, Clone)]
pub enum Entity {
    Description(String),
    Expression(Expr),
}

