#[derive(Clone, Debug, PartialEq)]
pub enum TokenStream {
    // Literais e identificadores
    Identifier(String),
    Bool(bool),
    Number(f64),
    String(String),
    // Palavras-chave
    And,
    Break,
    Do,
    Else,
    Elseif,
    End,
    False,
    For,
    Function,
    Goto,
    If,
    In,
    Local,
    Nil,
    Not,
    Or,
    Repeat,
    Return,
    Then,
    True,
    Until,
    While,

    // Operadores relacionais
    Equal,        // =
    EqualEqual,   // ==
    NotEqual,     // ~=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=

    // Operadores matematicos
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Caret,
    Hash,

    // Pontuação e estruturas
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Dot,
    DotDot,
    DotDotDot,
    Colon,
    Comma,
    Semicolon,

    // Fim do arquivo
    EOF,
}
