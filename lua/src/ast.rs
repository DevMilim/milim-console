use crate::{TokenStream, Value};

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    Identifier(String),
    BinaryOp {
        left: Box<Expr>,
        operator: TokenStream,
        right: Box<Expr>,
    },
    UnaryOp {
        operator: TokenStream,
        argument: Box<Expr>,
    },
    Call {
        target: Box<Expr>,
        args: Vec<Expr>,
    },
    Index {
        target: Box<Expr>,
        key: Box<Expr>,
    },

    Function {
        params: Vec<String>,
        body: Vec<Statement>,
    },
    LogicalOp {
        operator: TokenStream,
        argument: Box<Expr>,
    },
    MethodCall {
        target: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    VarArgs,
    Grouping(Box<Expr>),

    TableConstructor(Vec<TableField>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    LocalVar(String, Expr),
    Assign(Expr, Expr),
    If {
        condition: Expr,
        then_block: Vec<Statement>,
        else_block: Vec<Statement>,
    },
    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
    },
    While(Expr, Vec<Statement>),
    Break,
    Return(Option<Expr>),
    Expression(Expr),
    ForNumeric {
        counter: String,
        start: Expr,
        stop: Expr,
        step: Option<Expr>,
    },
    ForGeneric {
        vars: Vec<String>,
        iterators: Vec<Expr>,
        body: Vec<Statement>,
    },
    Do,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableField {
    Dynamic { key: Expr, value: Expr },
    Named { key: String, value: Expr },
    List(Expr),
}

#[derive(Clone, Debug)]
pub enum RuntimeError {
    AlreadyDeclared(String),
    AssignToConst(String),
    UndefinedVariable(String),
    InvalidOperation(String),
    Todo,
    InvalidBreak,
    InvalidContinue,
    NotCallable,
    InvalidAssignmentTarget,
    InvalidUnaryTarget,
    InvalidValue,
    Return(Value),
    Break,
}

pub fn apply_binary_op(
    left: Value,
    operator: &TokenStream,
    right: Value,
) -> Result<Value, RuntimeError> {
    match (left, operator, right) {
        // Matematicos
        (Value::Number(l), TokenStream::Plus, Value::Number(r)) => Ok(Value::Number(l + r)),
        (Value::Number(l), TokenStream::Minus, Value::Number(r)) => Ok(Value::Number(l - r)),
        (Value::Number(l), TokenStream::Asterisk, Value::Number(r)) => Ok(Value::Number(l * r)),
        (Value::Number(l), TokenStream::Slash, Value::Number(r)) => Ok(Value::Number(l / r)),
        (Value::Number(l), TokenStream::Caret, Value::Number(r)) => Ok(Value::Number(l.powf(r))),

        //Concatenação
        (l, TokenStream::DotDot, r) => {
            Ok(Value::String(format!("{}{}", l.to_string(), r.to_string())))
        }

        // Comparações
        (Value::Number(l), TokenStream::Greater, Value::Number(r)) => Ok(Value::Bool(l > r)),
        (Value::Number(l), TokenStream::Less, Value::Number(r)) => Ok(Value::Bool(l < r)),
        (Value::Number(l), TokenStream::EqualEqual, Value::Number(r)) => Ok(Value::Bool(l == r)),
        (Value::Number(l), TokenStream::NotEqual, Value::Number(r)) => Ok(Value::Bool(l != r)),

        // erro para tipos incompativeis ou todo
        (l, _, r) => Err(RuntimeError::InvalidOperation(format!(
            "não e possivel aplicar {:?} entre {:?} e {:?}",
            operator, l, r
        ))),
    }
}
