pub type Num = i64;


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
    pub face: Option<Num>,
    pub number: Num,
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
    Dice (Dice),
    Num (Num),
    Variable (String),
    Operator (Operator),
    Description(String),
}


#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Hash, Clone)]
pub struct Roll {
    pub expr: Expr,
    pub description: String,
    pub comparison: Option<Num>,
}


#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Hash, Clone)]
pub enum Command {
    Roll (Vec<Expr>),
    Unsupported,
    Say(String),
    Start,
}
