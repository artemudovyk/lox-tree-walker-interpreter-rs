use crate::runtime::{RuntimeError, Value};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub struct Env<'src> {
    map: HashMap<&'src str, Value<'src>>,
    outer: Option<Rc<RefCell<Env<'src>>>>,
}

impl<'src> Env<'src> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            outer: None,
        }
    }

    // TODO: same as new_with_outer
    pub fn from_closure(closure: Rc<RefCell<Self>>) -> Self {
        Self {
            map: HashMap::new(),
            outer: Some(closure),
        }
    }

    pub fn new_with_outer(outer: Rc<RefCell<Self>>) -> Self {
        Self {
            map: HashMap::new(),
            outer: Some(outer),
        }
    }

    fn to_outer(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }

    pub fn get(&self, key: &str) -> Option<Value<'src>> {
        self.map.get(key).cloned().or_else(|| {
            self.outer
                .as_ref()
                .and_then(|outer| outer.borrow().get(key))
        })
    }

    pub fn set(&mut self, key: &'src str, val: Value<'src>) {
        self.map.insert(key, val);
    }

    pub fn update(&mut self, key: &'src str, val: Value<'src>) -> Result<(), RuntimeError> {
        if !self.map.contains_key(key) {
            match &self.outer {
                Some(outer) => outer.borrow_mut().update(key, val)?,
                None => return Err(RuntimeError::UndeclaredVar(key.to_owned())),
            }
        } else {
            self.set(key, val);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_value_from_local_env_if_present() {
        let mut env = Env::new();
        env.set("test", Value::Bool(true));
        let res = env.get("test");
        assert_eq!(res, Some(Value::Bool(true)));
    }

    #[test]
    fn returns_value_from_outer_env_if_not_present_in_local() {
        let outer = Env::new().to_outer();
        outer.borrow_mut().set("test", Value::Bool(true));
        let inner = Env::new_with_outer(outer.clone());
        let res = inner.get("test");
        assert_eq!(res, Some(Value::Bool(true)));
    }

    #[test]
    fn update_changes_local_variable_but_not_overshadowed_from_outer() {
        let outer = Env::new().to_outer();
        outer.borrow_mut().set("test", Value::Number(1.0));
        let mut inner = Env::new_with_outer(outer.clone());
        inner.set("test", Value::Number(2.0));

        inner.update("test", Value::Number(3.0)).unwrap();

        assert_eq!(inner.get("test"), Some(Value::Number(3.0)));
        assert_eq!(outer.borrow().get("test"), Some(Value::Number(1.0)));
    }

    #[test]
    fn update_changes_outer_var_if_no_local_overshadowing_present() {
        let outer = Env::new().to_outer();
        outer.borrow_mut().set("test", Value::Number(1.0));
        let mut inner = Env::new_with_outer(outer.clone());

        inner.update("test", Value::Number(3.0)).unwrap();

        assert_eq!(outer.borrow().get("test"), Some(Value::Number(3.0)));
    }
}
