use crate::{Expr, Lexer, Statement, TableField, TokenStream};

#[derive(Clone, Debug, PartialEq)]
pub enum ParserErrors {
    NotNumber,
    UnexpectedToken(TokenStream),
    CurrentNull,
    PeekNull,
    ExpectedIdentifier,
    UnexpectedEOF,
    Todo,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub stmts: Vec<Statement>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Parser {
    tokens: Vec<TokenStream>,
    position: usize,
}
impl Parser {
    pub fn new(mut lex: Lexer) -> Self {
        Self {
            tokens: lex.lex_all(),
            position: 0,
        }
    }

    fn current(&self) -> Result<TokenStream, ParserErrors> {
        Ok(self
            .tokens
            .get(self.position)
            .ok_or(ParserErrors::CurrentNull))?
        .cloned()
    }
    fn peek(&self) -> Result<TokenStream, ParserErrors> {
        Ok(self
            .tokens
            .get(self.position + 1)
            .ok_or(ParserErrors::PeekNull))?
        .cloned()
    }
    fn advance(&mut self) {
        self.position += 1
    }
    fn expect(&mut self, expected: TokenStream) -> Result<(), ParserErrors> {
        let current = self.current()?;
        if current == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParserErrors::UnexpectedToken(current))
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParserErrors> {
        let mut stmts = Vec::new();

        while let Ok(token) = self.current() {
            if token == TokenStream::EOF {
                break;
            }
            stmts.push(self.parse_statement()?);
        }
        Ok(Program { stmts })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParserErrors> {
        let token = self.current()?;
        match token {
            TokenStream::Local => self.parse_local_var_func(),
            TokenStream::If => self.parse_if_statement(),
            TokenStream::Identifier(_) | TokenStream::LParen => self.parse_assignment_or_call(),
            TokenStream::Function => self.parse_function_def(),
            TokenStream::Return => self.parse_return(),
            TokenStream::While => self.parse_while(),
            TokenStream::Break => {
                self.advance();
                Ok(Statement::Break)
            }
            _ => self.parse_assignment_or_call(),
        }
    }
    fn parse_while(&mut self) -> Result<Statement, ParserErrors> {
        self.advance();

        let condition = self.parse_expression(0)?;

        self.expect(TokenStream::Do)?;
        let body = self.parse_block()?;

        self.expect(TokenStream::End)?;
        Ok(Statement::While(condition, body))
    }
    fn parse_return(&mut self) -> Result<Statement, ParserErrors> {
        self.advance();
        let token = self.current()?;
        if token == TokenStream::End
            || token == TokenStream::Else
            || token == TokenStream::Elseif
            || token == TokenStream::EOF
            || token == TokenStream::Semicolon
        {
            return Ok(Statement::Return(None));
        }

        let expr = self.parse_expression(0)?;
        Ok(Statement::Return(Some(expr)))
    }
    fn parse_function_def(&mut self) -> Result<Statement, ParserErrors> {
        self.advance();
        let name = match self.current()? {
            TokenStream::Identifier(id) => {
                self.advance();
                id
            }
            _ => return Err(ParserErrors::ExpectedIdentifier),
        };

        if self.current()? == TokenStream::Colon {
            self.advance();
            let method = match self.current()? {
                TokenStream::Identifier(id) => {
                    self.advance();
                    id
                }
                _ => return Err(ParserErrors::ExpectedIdentifier),
            };
            let (mut params, body) = self.parse_function_body()?;

            params.insert(0, "self".to_string());

            return Ok(Statement::Assign(
                Expr::Index {
                    target: Box::new(Expr::Identifier(name)),
                    key: Box::new(Expr::String(method)),
                },
                Expr::Function { params, body },
            ));
        }

        let (params, body) = self.parse_function_body()?;
        Ok(Statement::FunctionDef { name, params, body })
    }
    fn parse_function_body(&mut self) -> Result<(Vec<String>, Vec<Statement>), ParserErrors> {
        self.expect(TokenStream::LParen)?;

        let mut params = Vec::new();
        if self.current()? != TokenStream::RParen {
            loop {
                if let TokenStream::Identifier(id) = self.current()? {
                    params.push(id);
                    self.advance();
                } else {
                    return Err(ParserErrors::ExpectedIdentifier);
                }

                if self.current()? == TokenStream::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(TokenStream::RParen)?;

        let body = self.parse_block()?;
        self.expect(TokenStream::End)?;

        Ok((params, body))
    }
    fn parse_local_var(&mut self) -> Result<Statement, ParserErrors> {
        let name = match self.current()? {
            TokenStream::Identifier(id) => {
                self.advance();
                id
            }
            _ => return Err(ParserErrors::ExpectedIdentifier),
        };
        self.expect(TokenStream::Equal)?;

        let value = self.parse_expression(0)?;

        Ok(Statement::LocalVar(name, value))
    }

    fn parse_if_statement(&mut self) -> Result<Statement, ParserErrors> {
        self.advance();

        let condition = self.parse_expression(0)?;

        self.expect(TokenStream::Then)?;

        let then_block = self.parse_block()?;

        let mut elseif_chain = Vec::new();

        while let Ok(token) = self.current() {
            if token == TokenStream::Elseif {
                self.advance();
                let elseif_cond = self.parse_expression(0)?;
                self.expect(TokenStream::Then)?;
                let elseif_then = self.parse_block()?;
                let elseif_stmt = Statement::If {
                    condition: elseif_cond,
                    then_block: elseif_then,
                    else_block: Vec::new(),
                };
                elseif_chain.push(elseif_stmt);
                continue;
            }
            break;
        }

        let mut else_block = Vec::new();

        if self.current()? == TokenStream::Else {
            self.advance();
            let parsed_else = self.parse_block()?;
            if !elseif_chain.is_empty() {
                let mut last = elseif_chain.pop().unwrap();
                if let Statement::If {
                    condition,
                    then_block,
                    ..
                } = last
                {
                    last = Statement::If {
                        condition,
                        then_block,
                        else_block: parsed_else,
                    }
                }
                elseif_chain.push(last);
            } else {
                else_block = parsed_else
            }
        }

        if !elseif_chain.is_empty() {
            else_block = elseif_chain;
        }

        self.expect(TokenStream::End)?;
        Ok(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, ParserErrors> {
        let mut stmts = Vec::new();
        while let Ok(token) = self.current() {
            if token == TokenStream::End
                || token == TokenStream::Else
                || token == TokenStream::EOF
                || token == TokenStream::Elseif
            {
                break;
            }
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    pub fn parse_expression(&mut self, min_preced: i32) -> Result<Expr, ParserErrors> {
        let mut left = self.parse_unary()?;
        loop {
            let current_token = match self.current() {
                Ok(t) => t,
                Err(_) => break,
            };
            let preced = get_precedence(&current_token);
            if preced <= min_preced {
                break;
            }
            self.advance();

            let next_min_preced =
                if current_token == TokenStream::Caret || current_token == TokenStream::DotDot {
                    preced - 1
                } else {
                    preced
                };

            let right = self.parse_expression(next_min_preced)?;

            left = Expr::BinaryOp {
                left: Box::new(left),
                operator: current_token,
                right: Box::new(right),
            };
        }

        Ok(left)
    }
    fn parse_unary(&mut self) -> Result<Expr, ParserErrors> {
        let token = self.current()?;
        match token {
            TokenStream::Not | TokenStream::Hash | TokenStream::Minus => {
                self.advance();
                let operand = self.parse_expression(7)?;
                Ok(Expr::UnaryOp {
                    operator: token,
                    argument: Box::new(operand),
                })
            }
            _ => self.parse_primary(),
        }
    }
    fn parse_primary(&mut self) -> Result<Expr, ParserErrors> {
        let token = self.current()?;
        self.advance();
        use TokenStream::*;

        let mut expr = match token {
            Number(n) => Expr::Number(n),
            String(s) => Expr::String(s),
            True => Expr::Bool(true),
            False => Expr::Bool(false),
            Nil => Expr::Nil,
            Identifier(id) => Expr::Identifier(id),

            LParen => {
                let inner_expr = self.parse_expression(0)?;
                self.expect(RParen)?;
                inner_expr
            }
            LBrace => {
                let mut fields = Vec::new();
                while self.current()? != RBrace {
                    fields.push(self.parse_table_field()?);
                    if self.current()? == Comma || self.current()? == Semicolon {
                        self.advance();
                    }
                }
                self.expect(TokenStream::RBrace)?;
                Expr::TableConstructor(fields)
            }
            Function => {
                let (params, body) = self.parse_function_body()?;
                Expr::Function { params, body }
            }

            _ => return Err(ParserErrors::UnexpectedToken(token)),
        };
        loop {
            match self.current()? {
                LParen => {
                    self.advance();
                    let mut args = Vec::new();
                    if self.current()? != TokenStream::RParen {
                        loop {
                            args.push(self.parse_expression(0)?);
                            if self.current()? == TokenStream::Comma {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(TokenStream::RParen)?;
                    expr = Expr::Call {
                        target: Box::new(expr),
                        args,
                    }
                }
                Dot => {
                    self.advance();
                    let field = match self.current()? {
                        Identifier(id) => id,
                        _ => return Err(ParserErrors::ExpectedIdentifier),
                    };
                    self.advance();
                    expr = Expr::Index {
                        target: Box::new(expr),
                        key: Box::new(Expr::String(field)),
                    }
                }
                LBracket => {
                    self.advance();
                    let key_expr = self.parse_expression(0)?;
                    self.expect(TokenStream::RBracket)?;
                    expr = Expr::Index {
                        target: Box::new(expr),
                        key: Box::new(key_expr),
                    }
                }
                Colon => {
                    self.advance();
                    let method = match self.current()? {
                        Identifier(id) => id,
                        _ => return Err(ParserErrors::ExpectedIdentifier),
                    };

                    self.advance();
                    self.expect(LParen)?;
                    let mut args = Vec::new();
                    if self.current()? != RParen {
                        loop {
                            args.push(self.parse_expression(0)?);
                            if self.current()? == Comma {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(RParen)?;

                    expr = Expr::MethodCall {
                        target: Box::new(expr),
                        method,
                        args,
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }
    fn parse_local_var_func(&mut self) -> Result<Statement, ParserErrors> {
        self.advance();

        if self.current()? == TokenStream::Function {
            self.advance();
            let name = match self.current()? {
                TokenStream::Identifier(id) => {
                    self.advance();
                    id
                }
                _ => return Err(ParserErrors::ExpectedIdentifier),
            };
            let (params, body) = self.parse_function_body()?;
            Ok(Statement::FunctionDef { name, params, body })
        } else {
            self.parse_local_var()
        }
    }
    fn parse_assignment_or_call(&mut self) -> Result<Statement, ParserErrors> {
        let left = self.parse_expression(0)?;

        match self.current() {
            Ok(TokenStream::Equal) => {
                self.advance();
                let right = self.parse_expression(0)?;
                Ok(Statement::Assign(left, right))
            }

            _ => match left {
                Expr::Call { .. }
                | Expr::MethodCall { .. }
                | Expr::Index { .. }
                | Expr::Identifier(_) => Ok(Statement::Expression(left)),
                _ => Err(ParserErrors::UnexpectedToken(
                    self.current().unwrap_or(TokenStream::EOF),
                )),
            },
        }
    }
    fn parse_table_field(&mut self) -> Result<TableField, ParserErrors> {
        match self.current()? {
            TokenStream::LBracket => {
                self.advance();
                let key_expr = self.parse_expression(0)?;
                self.expect(TokenStream::RBracket)?;
                self.expect(TokenStream::Equal)?;
                let value_expr = self.parse_expression(0)?;
                Ok(TableField::Dynamic {
                    key: key_expr,
                    value: value_expr,
                })
            }
            TokenStream::Identifier(id) => {
                if let Ok(TokenStream::Equal) = self.peek() {
                    self.advance();
                    self.advance();
                    let value_expr = self.parse_expression(0)?;
                    Ok(TableField::Named {
                        key: id,
                        value: value_expr,
                    })
                } else {
                    let expr = self.parse_expression(0)?;
                    Ok(TableField::List(expr))
                }
            }
            _ => {
                let expr = self.parse_expression(0)?;
                Ok(TableField::List(expr))
            }
        }
    }
}

fn get_precedence(token: &TokenStream) -> i32 {
    use TokenStream::*;
    match token {
        Or => 1,
        And => 2,
        EqualEqual | NotEqual | Less | LessEqual | Greater | GreaterEqual => 3,
        DotDot => 4,
        Plus | Minus => 5,
        Asterisk | Slash | Percent => 6,
        Caret => 8,
        _ => 0,
    }
}
