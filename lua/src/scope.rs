use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{RuntimeError, Value};

pub type EnvRef = Rc<RefCell<Env>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Env {
    vars: HashMap<String, Binding>,
    parent: Option<EnvRef>,
}

impl Env {
    pub fn new() -> EnvRef {
        Rc::new(RefCell::new(Self {
            vars: HashMap::new(),
            parent: None,
        }))
    }
    pub fn with_parent(parent: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Self {
            vars: HashMap::new(),
            parent: Some(parent),
        }))
    }
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.vars.get(name) {
            Some(v.value.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }
    pub fn assign(&mut self, name: String, value: Value) -> Result<(), RuntimeError> {
        if let Some(b) = self.vars.get_mut(&name) {
            if !b.mutable {
                return Err(RuntimeError::AssignToConst(name));
            }
            b.value = value;
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().assign(name, value)
        } else {
            Err(RuntimeError::UndefinedVariable(name))
        }
    }
    pub fn declare(
        &mut self,
        name: String,
        value: Value,
        mutable: bool,
    ) -> Result<(), RuntimeError> {
        if self.vars.contains_key(&name) {
            return Err(RuntimeError::AlreadyDeclared(name));
        }
        self.vars.insert(name, Binding { value, mutable });
        Ok(())
    }
}

impl NativeImpl for Rc<RefCell<Env>> {
    fn native_fn(&self, name: &str, func: fn(&[Value]) -> Value) -> Result<(), RuntimeError> {
        self.borrow_mut()
            .declare(name.to_owned(), Value::NativeFuncion(func), false)
    }
}

pub trait NativeImpl {
    fn native_fn(&self, name: &str, func: fn(&[Value]) -> Value) -> Result<(), RuntimeError>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    pub value: Value,
    pub mutable: bool,
}
