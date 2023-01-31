use lazy_static::__Deref;

use super::Value;
use std::collections::HashMap;

pub trait NameSpace {
    fn root() -> Self;

    fn get(&self, x: &str) -> Option<&Value>;

    fn add(&mut self, x: String, v: Value) -> Result<(), String>;
    //fonction ajouter une variable mutable
    fn is_mutable(&self, x: &str) -> bool;
    fn add_mutable(&mut self, x: String, v: Value) -> Result<(), String>;
    fn mutate(&mut self, x: String, v: Value) -> Result<(), String>;

    fn enter_block(&mut self) -> ();

    fn exit_block(&mut self) -> Result<(), String>;
}

pub struct VNameSpace {
    stack: Vec<HashMap<String, Value>>,
    variablesMutables: Vec<Vec<String>>,//stock les varibles mutables de chaque block
}

impl NameSpace for VNameSpace {
    fn root() -> VNameSpace {
        return VNameSpace {
            stack: vec![HashMap::new()],
            variablesMutables: vec![Vec::new()],
        };
    }

    fn get(&self, x: &str) -> Option<&Value> {
        for i in self.stack.iter().rev() {
            if i.contains_key(x) {
                return i.get(x);
            }
        }
        return None;
    }

    fn add(&mut self, x: String, v: Value) -> Result<(), String> {
        if self.stack.last().unwrap().contains_key(&x) {
            return Err(format!(" {}  is already defined.", x));
        } else {
            self.stack.last_mut().unwrap().insert(x, v);
            Ok(())
        }
    }
    fn is_mutable(&self, x: &str) -> bool {
        for i in self.variablesMutables.iter() {
            for j in i.iter() {
                if j == x {
                    return true;
                }
            }
        }
        return false;
    }
    fn add_mutable(&mut self, x: String, v: Value) -> Result<(), String> {
        if self.stack.last().unwrap().contains_key(&x) {
            return Err(format!(" {}  is already defined.", x));
        } else {
            self.stack.last_mut().unwrap().insert(x.clone(), v);
            self.variablesMutables.last_mut().unwrap().push(x);
            Ok(())
        }
    }
    fn mutate(&mut self, x: String, v: Value) -> Result<(), String> {
        if !self.is_mutable(&x) {
            return Err(format!(" {}  is not mutable.", x));
        } else if !self.get(&x).is_none() {
            for i in self.stack.iter_mut(){
                if i.contains_key(x.as_str()){
                    // i want to change the value of the key
                    i.insert(x, v);//modify the value of the key x
                    return Ok(());
                }
            }
            Ok(())
        } else {
            return Err(format!(" {}  is not defined.", x));
        }
    }
    fn enter_block(&mut self) {
        self.stack.push(HashMap::new());
    }

    fn exit_block(&mut self) -> Result<(), String> {
        if self.stack.len() == 1 {
            return Err("Cannot exit root block.".to_string());
        }
        
        self.stack.pop();
        Ok(())
    }
}
