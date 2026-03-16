use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::Chunk;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(f64),
    String(String),
    Function(Rc<Function>),
    Nil,
    #[warn(unpredictable_function_pointer_comparisons)]
    NativeFuncion(fn(&[Value]) -> Value),
    Table(Rc<RefCell<HashMap<Value, Value>>>),
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        enum SerializableValue<'a> {
            Bool(bool),
            Number(f64),
            String(&'a String),
            Function(&'a Function),
            Nil,
        }
        match self {
            Value::Bool(b) => SerializableValue::Bool(*b).serialize(serializer),
            Value::Number(n) => SerializableValue::Number(*n).serialize(serializer),
            Value::String(s) => SerializableValue::String(s).serialize(serializer),
            Value::Function(f) => SerializableValue::Function(f).serialize(serializer),
            Value::Nil => SerializableValue::Nil.serialize(serializer),
            Value::NativeFuncion(_) | Value::Table(_) => {
                Err(serde::ser::Error::custom(
                    "Não e possivel serializar objetos de runtime",
                ))
            }
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        enum SerializedValue {
            Bool(bool),
            Number(f64),
            String(String),
            Function(Function),
            Nil,
        }
        let helper = SerializedValue::deserialize(deserializer)?;

        Ok(match helper {
            SerializedValue::Bool(b) => Value::Bool(b),
            SerializedValue::Number(n) => Value::Number(n),
            SerializedValue::String(s) => Value::String(s),
            SerializedValue::Function(f) => Value::Function(Rc::new(f)),
            SerializedValue::Nil => Value::Nil,
        })
    }
}
impl Eq for Value {}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    pub chunk: Chunk,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match *self {
            Value::Bool(b) => b,
            Value::Nil => false,
            _ => true,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Nil => "nil".to_string(),
            Value::Function(_) => "".to_string(),
            Value::NativeFuncion(_) => "".to_string(),
            Value::Table(_) => "".to_string(),
        }
    }
}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Value::Bool(b) => b.hash(state),
            Value::Number(n) => n.to_bits().hash(state),
            Value::String(s) => s.hash(state),
            Value::Nil => 0.hash(state),
            Value::Function(f) => f.hash(state),
            Value::NativeFuncion(f) => (*f as usize).hash(state),
            Value::Table(t) => Rc::as_ptr(t).hash(state),
        }
    }
}
