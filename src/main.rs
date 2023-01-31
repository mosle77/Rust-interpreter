extern crate lazy_static;
extern crate pest;
extern crate pest_derive;

use std::io::Write;
use std::io::{self, BufRead};

mod ast;
use ast::Expression;
use ast::Instruction;
use ast::Value;
use Expression::*;
use Instruction::*;

mod parser;
use parser::parse;

mod namespace;
use namespace::NameSpace;
use namespace::VNameSpace;

fn eval_expr<NS: NameSpace>(expr: &Expression, ns: &NS) -> Result<Value, String> {
    match expr {
        Const(n) => Ok(*n),
        //let x=1 
        //x 
        //Value= 1
        Var(s) => Ok(*ns.get(s).unwrap()),
        BinOp(exp1, op, exp2) => {
            let left = eval_expr(exp1, ns)?;
            let right = eval_expr(exp2, ns)?;
            let op = op.as_str();//convert to str
            match (left, op, right) {
                // Integers
                (Value::Integer(l), "+", Value::Integer(r)) => Ok(Value::Integer(l + r)),
                (Value::Integer(l), "*", Value::Integer(r)) => Ok(Value::Integer(l * r)),
                (Value::Integer(l), "-", Value::Integer(r)) => Ok(Value::Integer(l - r)),
                (Value::Integer(l), "/", Value::Integer(r)) => Ok(Value::Integer(l / r)),
                (Value::Integer(l), "%", Value::Integer(r)) => Ok(Value::Integer(l % r)),
                (Value::Integer(l), "<", Value::Integer(r)) => Ok(Value::Boolean(l < r)),
                (Value::Integer(l), ">", Value::Integer(r)) => Ok(Value::Boolean(l > r)),
                (Value::Integer(l), "<=", Value::Integer(r)) => Ok(Value::Boolean(l <= r)),
                (Value::Integer(l), ">=", Value::Integer(r)) => Ok(Value::Boolean(l >= r)),
                (Value::Integer(l), "==", Value::Integer(r)) => Ok(Value::Boolean(l == r)),
                (Value::Integer(l), "!=", Value::Integer(r)) => Ok(Value::Boolean(l != r)),
                // Booleans
                (Value::Boolean(l), "&&", Value::Boolean(r)) => Ok(Value::Boolean(l && r)),
                (Value::Boolean(l), "||", Value::Boolean(r)) => Ok(Value::Boolean(l || r)),
                (Value::Boolean(l), "==", Value::Boolean(r)) => Ok(Value::Boolean(l == r)),
                (Value::Boolean(l), "!=", Value::Boolean(r)) => Ok(Value::Boolean(l != r)),
                // invalid operation
                _ => {
                    return Err(format!(
                        "invalid operation: {:?} {:?} {:?}",
                        left, op, right
                    ))
                }
            }
        }
    }
}

fn eval<NS: NameSpace>(instruction: &Instruction, ns: &mut NS) -> Result<Value, String> {
    match instruction {
        Expr(exp) =>eval_expr(exp, ns),
        // if cond {cond_true} else {cond_false}
        IfElse {
            cond,
            cond_true,
            cond_false,
        } => match eval_expr(cond, ns) {
            Ok(Value::Boolean(b)) => {
                if b {
                    match eval(cond_true, ns) {
                        Ok(v) => Ok(v),
                        Err(e) => Err(e),
                    }
                } else {
                    match eval(cond_false, ns) {
                        Ok(v) => Ok(v),
                        Err(e) => Err(e),
                    }
                }
            }
            _ => Err("Expected a boolean value for the condition.".to_string()),
        },
        Let(x, e) => {
            let v = eval_expr(e, ns)?;
            let result = ns.add(x.to_string(), v);
            match result {
                Ok(_) => Ok(v),
                Err(e) => Err(e),
            }
        }
        LetMut(x, e) => {
            let v = eval_expr(e, ns)?;
            let result = ns.add_mutable(x.to_string(), v);
            match result {
                Ok(_) => Ok(v),
                Err(e) => Err(e),
            }
        }
        Mutate(x, e) => {
            let v = eval_expr(e, ns)?;
            let result = ns.mutate(x.to_string(), v);
            // print!("mutate {} = {}\n", x, v);
            match result {
                Ok(_) => {
                    Ok(v)
                }
                Err(e) => Err(e),
            }
        }
        While(cond, body) => {
            // loop while the condition is true
            let mut i = true;
            while i {
                // evaluate the body of the loop
                match eval_expr(cond, ns) {
                    Ok(Value::Boolean(true)) => match eval(body, ns) {
                        Ok(v) => {
                        }
                        Err(e) => return Err(e),
                    },
                    Ok(Value::Boolean(false)) => {
                        i = false;
                    }
                    _ => {
                        return Err("Expected a boolean value for the condition.".to_string());
                    }
                }
            }
            Ok(Value::Unit)
        }
        Block(instructions) => {
            // get first instruction
            ns.enter_block();
            for i in instructions {
                match eval(i, ns) {
                    // EVAL
                    Err(msg) => {
                        print!("Evaluation Error : {}\n", msg)
                    }
                    Ok(v) => {
                        print!(
                            "{} : {} = {}\n",
                            id_of_instruction(&instruction),
                            type_of_value(v),
                            v
                        )
                    } // PRINT
                }
            }
            let result = ns.exit_block();
            match result {
                Ok(_) => Ok(Value::Unit),
                Err(e) => Err(e),
            }
        }
    }
}

fn prompt() {
    print!("imp # ");
    io::stdout().flush().unwrap();
}

pub fn id_of_instruction(ast: &Instruction) -> String {
    match ast {
        Let(x, _) => x.to_string(),
        Expr(_) => "-".to_string(),
        Block(_) => "-".to_string(),
        LetMut(x, _) => x.to_string(),
        Mutate(x, _) => x.to_string(),
        While(_, _) => "-".to_string(),
        IfElse {
            cond,
            cond_true,
            cond_false,
        } => "-".to_string(),
    }
}

fn type_of_value(v: Value) -> String {
    match v {
        Value::Integer(_) => "int".to_string(),
        Value::Unit => "unit".to_string(),
        Value::Boolean(_) => "bool".to_string(),
    }
}

fn main() {
    let mut ns = VNameSpace::root();
    ns.enter_block();
    prompt();
    for line in io::stdin().lock().lines() {
        let input = line.unwrap(); // READ
        match parse(input) {
            // PARSE
            Err(msg) => {
                print!("Parse Error: {}\n", msg)
            }
            Ok(instruction) => {
                match eval(&instruction, &mut ns) {
                    // EVAL
                    Err(msg) => {
                        print!("Evaluation Error : {}\n", msg)
                    }
                    Ok(v) => {
                        print!(
                            "{} : {} = {}\n",
                            id_of_instruction(&instruction),
                            type_of_value(v),
                            v
                        )
                    } // PRINT
                }
            }
        };
        prompt()
    }
}
