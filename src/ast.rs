use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Integer(i32),
    Boolean(bool),
    Unit,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Unit => write!(f, "()"),
        }
    }
}

#[derive(Debug)]
pub enum Expression {
    Const(Value),
    Var(String),
    BinOp(Box<Expression>, String, Box<Expression>),
}

#[derive(Debug)]
pub enum Instruction {
    Expr(Expression),
    IfElse {
        cond: Expression,
        cond_true: Box<Instruction>,//block of instructions si la condition est vraie
        cond_false: Box<Instruction>,//block of instructions si la condition est false
    },
    Let(String, Expression),
    LetMut(String, Expression),
    Mutate(String, Expression),
    While(Expression, Box<Instruction>),
    Block(Vec<Instruction>),
}
