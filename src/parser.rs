// see https://pest.rs/book/ and https://pest.rs/book/examples/calculator.html
use pest::iterators::Pairs;
use pest::pratt_parser::PrattParser;
use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
pub struct MyParser;

use super::Expression;
use super::Instruction;
use super::Value;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            // Addition and subtract have equal precedence
            .op(Op::infix(EQQUALS, Left) | Op::infix(LEQ, Left) | Op::infix(GEQ, Left) | Op::infix(LOWER, Left) | Op::infix(GREATER, Left))
            .op(Op::infix(ADD, Left) | Op::infix(SUBTRACT, Left))
            .op(Op::infix(MULTIPLY, Left) | Op::infix(DIVIDE, Left) | Op::infix(MODULO, Left))
    };
}

fn parse_expr(pairs: Pairs<Rule>) -> Expression {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::integer => {
                Expression::Const(Value::Integer(primary.as_str().parse::<i32>().unwrap()))
            }
            Rule::identifier => Expression::Var(primary.as_str().to_string()),
            Rule::expr => parse_expr(primary.into_inner()),
            Rule::TRUE => Expression::Const(Value::Boolean(true)),
            Rule::FALSE => Expression::Const(Value::Boolean(false)),
            rule => unreachable!("parse_expr expected atom, found {:?}", rule),
        })
        .map_infix(|lhs, op, rhs| {
            let op2 = op.as_span().as_str().to_string();
            Expression::BinOp(Box::new(lhs), op2, Box::new(rhs))
        })
        .parse(pairs)
}

fn parse_block(mut pairs: Pairs<Rule>) -> Result<Vec<Instruction>, String> {
    let first_rule = pairs.next().unwrap();
    let mut res = vec![];
    match first_rule.as_rule() {
        Rule::empty_block => {}
        Rule::non_empty_block => {
            let mut rules = first_rule.into_inner();
            while let Some(rule) = rules.next() {
                if rule.as_rule() == Rule::instr {
                    let instr = parse_instr(rule.into_inner())?;
                    // print!("{:?} ", instr);
                    res.push(instr)
                }
            }
        }
        _ => unreachable!("parse_block expected instrs, found {:?}", first_rule),
    };
    Ok(res)
}

fn parse_instr(mut pairs: Pairs<Rule>) -> Result<Instruction, String> {
    let first_rule = pairs.next().unwrap();
    match first_rule.as_rule() {
        Rule::expr => Ok(Instruction::Expr(parse_expr(first_rule.into_inner()))),
        Rule::let_equals => {
            let mut rules = first_rule.into_inner();
            let id = rules.next().unwrap().as_span().as_str().to_string();
            let expr = rules.next().unwrap().into_inner();
            Ok(Instruction::Let(id, parse_expr(expr)))
        }
        Rule::let_mut_equals => {
            let mut rules = first_rule.into_inner();
            let id = rules.next().unwrap().as_span().as_str().to_string();
            let expr = rules.next().unwrap().into_inner();
            Ok(Instruction::LetMut(id, parse_expr(expr)))
        }
        Rule::instrs => Ok(Instruction::Block(parse_block(first_rule.into_inner())?)),
        Rule::if_instr => {
            let mut rules = first_rule.into_inner();
            let cond = parse_expr(rules.next().unwrap().into_inner());
            let cond_true = Box::new(Instruction::Block(parse_block(
                rules.next().unwrap().into_inner(),
            )?));
            let cond_false = Box::new(Instruction::Block(parse_block(
                rules.next().unwrap().into_inner(),
            )?));
            Ok(Instruction::IfElse {
                cond,
                cond_true,
                cond_false,
            })
        }
        Rule::while_instr => {
            let mut rules = first_rule.into_inner();
            let cond = parse_expr(rules.next().unwrap().into_inner());
            let body = Box::new(Instruction::Block(parse_block(
                rules.next().unwrap().into_inner(),
            )?));
            Ok(Instruction::While(cond, body))
        }
        Rule::mutate_instr => {
            let mut rules = first_rule.into_inner();
            let id = rules.next().unwrap().as_span().as_str().to_string();
            let expr = parse_expr(rules.next().unwrap().into_inner());
            Ok(Instruction::Mutate(id, expr))
        }
        _ => unreachable!("parse_instr expected instr, found {:?}", first_rule),
    }
}

pub fn parse(input: String) -> Result<Instruction, String> {
    match MyParser::parse(Rule::start_rule, &input) {
        Ok(mut pairs) => {
            let first_rule = pairs.next().unwrap();
            match first_rule.as_rule() {
                Rule::instr => parse_instr(first_rule.into_inner()),
                _ => {
                    panic!("the grammar is not as expected")
                }
            }
        }
        Err(e) => Err(format!("{:?}", e)),
    }
}

// TESTS

mod tests {

    use super::*;

    #[test]
    fn test() {
        let mut examples: Vec<&str> = vec![];
        examples.push("1+1");
        examples.push("6 % 3 == 1 - 2 / 2");
        examples.push("(0 + 1) * true");
        examples.push("(0 > 1) == true");
        examples.push("true + 1");
        examples.push("if (true) {1} else {2}");
        examples.push("if true {if false {0} else {1}} else {2}");
        examples.push("let x = true");
        examples.push("let x = false");
        examples.push("x");
        examples.push("let y = 10");
        examples.push("if x {let x = 1; {let x=2;x}; x+y} else {x}");
        examples.push("let n = 6");
        examples.push("let mut acc = 1");
        examples.push("let mut i = 1");
        examples.push("i = 2");
        examples.push("while i <= n {acc = acc * i; i = i+1}");

        for s in examples {
            let instr = parse(s.to_string()).expect(&format!("{} not parsed", s));
            print!("{}\n{:?}\n\n", s, instr)
        }
    }
}
