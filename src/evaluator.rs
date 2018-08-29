use types::*;
use rand::{Rng, thread_rng};
use failure::Error;


const MAX_DICE: Num = 999;
const MAX_FACE: Num = 10000;


pub struct Context {
    pub default_face: Num,
}

impl Context {
    pub fn new() -> Context {
        Context {
            default_face: 100,
        }
    }

    pub fn roll(&self, n: Option<Num>) -> Num {
        if Some(1) == n {
            return 1;
        }
        let n = n.unwrap_or(self.default_face);
        thread_rng().gen_range(1, n+1)
    }
}


fn dice_roll(context: &Context, dice: Dice) -> Result<Vec<Num>, Error> {
    if let Some(face) = dice.face {
        if face > MAX_FACE {
            return Err(format_err!("too much dice face (max {})", MAX_FACE));
        }
    }
    if dice.number > 999 {
        return Err(format_err!("too much dice (max {})", MAX_DICE));
    }
    Ok((0..dice.number).map(|_| context.roll(dice.face)).collect())
}


pub fn eval_roll(context: &Context, roll: Vec<Expr>) -> Result<(Num, String), Error> {
    let mut value: Num = 0;
    let mut op_stack: Vec<Operator> = vec![Operator::Add];
    let mut log: Vec<String> = vec![];

    for i in 0..roll.len() {
        let expr = roll[i].clone();
        let mut xs: Vec<Num> = match expr {
            Expr::Num(x) => vec![x],
            Expr::Dice(dice) => dice_roll(context, dice)?,
            Expr::Operator(op) => {
                op_stack.push(op);
                log.push(op.show().to_string());
                continue
            }
            Expr::Description(desc) => {
                log.push(desc);
                continue
            },
            Expr::Variable(var) => {
                log.push(format!(".{}(unsupported)", var));
                continue
            }
        };

        while let Some(op) = op_stack.pop() {
            if xs.len() > 1 {
                log.push(format!("{:?} =", xs));
            }
            match op {
                Operator::Max => xs = xs.into_iter().max().into_iter().collect(),
                Operator::Min => xs = xs.into_iter().min().into_iter().collect(),
                operator => {
                    let x: Num = xs.into_iter().sum();
                    log.push(format!("{}", x));
                    value = match operator {
                        Operator::Add => value.checked_add(x),
                        Operator::Sub => value.checked_sub(x),
                        Operator::Mul => value.checked_mul(x),
                        Operator::Div => value.checked_div(x),
                        _ => unreachable!(),
                    }.ok_or(format_err!("arithmetical error"))?;
                    break
                }
            }
        }
        op_stack.clear();
        op_stack.push(Operator::Add);
    }
    return Ok((value, log.join(" ")));
}
