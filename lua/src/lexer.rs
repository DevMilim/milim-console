use crate::TokenStream;

#[derive(Clone, Debug)]
pub struct Lexer {
    chars: Vec<char>,
    position: usize,
}
#[derive(Clone, Debug)]
pub enum LexerError {
    ParseIntError,
    FirstCharIdentNotAccept,
    InvalidChar(char),
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let input: Vec<char> = input.chars().collect();
        Self {
            chars: input,
            position: 0,
        }
    }
    pub fn lex_all(&mut self) -> Vec<TokenStream> {
        let mut tokens = Vec::new();
        loop {
            let current = self.next_token();
            match current {
                Ok(value) => {
                    tokens.push(value.clone());
                    if value == TokenStream::EOF {
                        break;
                    }
                }
                Err(err) => {
                    eprintln!("{:#?}", err);
                    break;
                }
            }
        }
        tokens
    }
    pub fn next_token(&mut self) -> Result<TokenStream, LexerError> {
        self.skip_whitespace();
        if let Some(ch) = self.current() {
            use TokenStream::*;
            match ch {
                c if c.is_alphabetic() || c == '_' => {
                    let ident = self.read_identifier()?;

                    let token = match ident.as_str() {
                        "and" => And,
                        "break" => Break,
                        "do" => Do,
                        "else" => Else,
                        "elseif" => Elseif,
                        "end" => End,
                        "for" => For,
                        "function" => Function,
                        "goto" => Goto,
                        "if" => If,
                        "in" => In,
                        "local" => Local,
                        "nil" => Nil,
                        "not" => Not,
                        "or" => Or,
                        "repeat" => Repeat,
                        "return" => Return,
                        "then" => Then,
                        "true" => True,
                        "false" => False,
                        "until" => Until,
                        "while" => While,
                        _ => Identifier(ident),
                    };

                    Ok(token)
                }

                c if c.is_numeric() => {
                    let value = self.read_float()?;
                    Ok(Number(value))
                }
                c if c == '"' => {
                    let value = self.read_string();
                    Ok(String(value))
                }
                '(' => {
                    self.advance();
                    Ok(LParen)
                }
                ')' => {
                    self.advance();
                    Ok(RParen)
                }
                '{' => {
                    self.advance();
                    Ok(LBrace)
                }
                '}' => {
                    self.advance();
                    Ok(RBrace)
                }
                '[' => {
                    self.advance();
                    Ok(LBracket)
                }
                ']' => {
                    self.advance();
                    Ok(RBracket)
                }
                ':' => {
                    self.advance();
                    Ok(Colon)
                }
                ',' => {
                    self.advance();
                    Ok(Comma)
                }
                ';' => {
                    self.advance();
                    Ok(Semicolon)
                }
                '*' => {
                    self.advance();
                    Ok(Asterisk)
                }
                '+' => {
                    self.advance();
                    Ok(Plus)
                }
                '-' => {
                    if self.peek() == Some(&'-') {
                        self.advance();
                        self.advance();
                        if self.current() == Some('[') && self.peek() == Some(&'[') {
                            self.advance();
                            self.advance();
                            self.skip_multiline_comment();
                        } else {
                            self.skip_single_line_comment();
                        }
                        self.next_token()
                    } else {
                        self.advance();
                        Ok(Minus)
                    }
                }
                '/' => {
                    self.advance();
                    Ok(Slash)
                }
                '%' => {
                    self.advance();
                    Ok(Percent)
                }
                '^' => {
                    self.advance();
                    Ok(Caret)
                }
                '#' => {
                    self.advance();
                    Ok(Hash)
                }
                '=' => {
                    if self.peek() == Some(&'=') {
                        self.advance();
                        self.advance();
                        Ok(EqualEqual)
                    } else {
                        self.advance();
                        Ok(Equal)
                    }
                }
                '~' => {
                    if self.peek() == Some(&'=') {
                        self.advance();
                        self.advance();
                        Ok(NotEqual)
                    } else {
                        self.advance();
                        Err(LexerError::InvalidChar(ch))
                    }
                }
                '<' => {
                    if self.peek() == Some(&'=') {
                        self.advance();
                        self.advance();
                        Ok(LessEqual)
                    } else {
                        self.advance();
                        Ok(Less)
                    }
                }
                '>' => {
                    if self.peek() == Some(&'=') {
                        self.advance();
                        self.advance();
                        Ok(GreaterEqual)
                    } else {
                        self.advance();
                        Ok(Greater)
                    }
                }
                '.' => {
                    if self.peek() == Some(&'.') {
                        self.advance();
                        if self.peek() == Some(&'.') {
                            self.advance();
                            self.advance();
                            Ok(DotDotDot)
                        } else {
                            self.advance();
                            Ok(DotDot)
                        }
                    } else {
                        self.advance();
                        Ok(Dot)
                    }
                }

                _ => Err(LexerError::InvalidChar(ch)),
            }
        } else {
            Ok(TokenStream::EOF)
        }
    }

    fn peek(&self) -> Option<&char> {
        self.chars.get(self.position + 1)
    }

    fn current(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }
    fn advance(&mut self) {
        self.position += 1
    }
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    fn read_identifier(&mut self) -> Result<String, LexerError> {
        let mut ident = String::new();
        if let Some(ch) = self.current() {
            if ch.is_alphabetic() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                return Err(LexerError::FirstCharIdentNotAccept);
            }
        }
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        Ok(ident)
    }
    fn read_float(&mut self) -> Result<f64, LexerError> {
        let mut int = String::new();
        while let Some(ch) = self.current() {
            if ch.is_numeric() {
                int.push(ch);
                self.advance();
            } else if ch == '.' {
                if self.peek() == Some(&'.') {
                    break;
                }
                self.advance();
                int.push(ch);
            } else {
                break;
            }
        }
        match int.parse::<f64>() {
            Ok(value) => Ok(value),
            Err(_) => Err(LexerError::ParseIntError),
        }
    }
    fn read_string(&mut self) -> String {
        let mut value = String::new();
        self.advance();
        while let Some(ch) = self.current() {
            if ch == '"' {
                self.advance();
                break;
            } else {
                value.push(ch);
                self.advance();
            }
        }
        value
    }
    fn skip_single_line_comment(&mut self) {
        while let Some(ch) = self.current() {
            if ch == '\n' {
                self.advance();
                break;
            }
            self.advance();
        }
    }
    fn skip_multiline_comment(&mut self) {
        while let Some(ch) = self.current() {
            if ch == ']' && self.peek() == Some(&']') {
                self.advance();
                self.advance();
                break;
            }
            self.advance();
        }
    }
}
