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

                    return Ok(token);
                }

                c if c.is_numeric() => {
                    let value = self.read_float()?;
                    return Ok(Number(value));
                }
                c if c == '"' => {
                    let value = self.read_string();
                    return Ok(String(value));
                }
                '(' => {
                    self.advance();
                    return Ok(LParen);
                }
                ')' => {
                    self.advance();
                    return Ok(RParen);
                }
                '{' => {
                    self.advance();
                    return Ok(LBrace);
                }
                '}' => {
                    self.advance();
                    return Ok(RBrace);
                }
                '[' => {
                    self.advance();
                    return Ok(LBracket);
                }
                ']' => {
                    self.advance();
                    return Ok(RBracket);
                }
                ':' => {
                    self.advance();
                    return Ok(Colon);
                }
                ',' => {
                    self.advance();
                    return Ok(Comma);
                }
                ';' => {
                    self.advance();
                    return Ok(Semicolon);
                }
                '*' => {
                    self.advance();
                    return Ok(Asterisk);
                }
                '+' => {
                    self.advance();
                    return Ok(Plus);
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
                        return self.next_token();
                    } else {
                        self.advance();
                        return Ok(Minus);
                    }
                }
                '/' => {
                    self.advance();
                    return Ok(Slash);
                }
                '%' => {
                    self.advance();
                    return Ok(Percent);
                }
                '^' => {
                    self.advance();
                    return Ok(Caret);
                }
                '#' => {
                    self.advance();
                    return Ok(Hash);
                }
                '=' => {
                    if self.peek() == Some(&'=') {
                        self.advance();
                        self.advance();
                        return Ok(EqualEqual);
                    } else {
                        self.advance();
                        return Ok(Equal);
                    }
                }
                '~' => {
                    if self.peek() == Some(&'=') {
                        self.advance();
                        self.advance();
                        return Ok(NotEqual);
                    } else {
                        self.advance();
                        return Err(LexerError::InvalidChar(ch));
                    }
                }
                '<' => {
                    if self.peek() == Some(&'=') {
                        self.advance();
                        self.advance();
                        return Ok(LessEqual);
                    } else {
                        self.advance();
                        return Ok(Less);
                    }
                }
                '>' => {
                    if self.peek() == Some(&'=') {
                        self.advance();
                        self.advance();
                        return Ok(GreaterEqual);
                    } else {
                        self.advance();
                        return Ok(Greater);
                    }
                }
                '.' => {
                    if self.peek() == Some(&'.') {
                        self.advance();
                        if self.peek() == Some(&'.') {
                            self.advance();
                            self.advance();
                            return Ok(DotDotDot);
                        } else {
                            self.advance();
                            return Ok(DotDot);
                        }
                    } else {
                        self.advance();
                        return Ok(Dot);
                    }
                }

                _ => return Err(LexerError::InvalidChar(ch)),
            }
        } else {
            return Ok(TokenStream::EOF);
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
