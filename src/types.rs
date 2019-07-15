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

impl Dice {
    pub fn show(&self) -> String {
        if let Some(face) = self.face {
            format!("{}d{}", self.number, face)
        }
        else {
            format!("{}d", self.number)
        }
    }
}


#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Hash, Clone)]
pub enum Expr {
    Prefix(Operator, Box<Expr>),
    Infix(Box<Expr>, Operator, Box<Expr>),
    Num(Int),
    Roll(Dice),
    Child(Box<Expr>),
}

impl Expr {
    pub fn show(&self) -> String {
        use Expr::*;

        match self {
            Prefix(op, e) => format!("{} {}", op.show(), e.show()),
            Infix(l, op, r) => format!("{} {} {}", l.show(), op.show(), r.show()),
            Num(n) => format!("{}", n),
            Roll(dice) => dice.show(),
            Child(expr) => format!("({})", expr.show()),
        }
    }
}


#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Hash, Clone)]
pub enum Entity {
    Description(String),
    Expression(Expr),
}

impl Entity {
    pub fn show(&self) -> String {
        use Entity::*;

        match self {
            Description(s) => s.clone(),
            Expression(e) => e.show(),
        }
    }
}

