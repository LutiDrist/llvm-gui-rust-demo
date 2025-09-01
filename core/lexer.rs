#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(i64),
    Ident(String),

   Fn,
   Let,
   If,
   Else,
   While,

    LBrace,
    RBrace,
    LParen,
    RParen,
    Semicolon,
    Eq,

    Plus,
    Minus,
    Star,
    Slash,

    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,

    Error,
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self{ input, pos: 0}
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.input[self.pos..].chars().next()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }


    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
 
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek_char() {
            match ch
                 {
                ' ' | '\n' | '\t' | '\r' => {
                    self.next_char();
                }
                '+' => {
                    self.next_char();
                    tokens.push(Token::Plus);
                }
                '-' => {
                    self.next_char();
                    tokens.push(Token::Minus);
                }
                '(' => {
                    self.next_char();
                    tokens.push(Token::LParen);
                }
                ')' => {
                    self.next_char();
                    tokens.push(Token::RParen);
                }
                '{' => {
                    self.next_char();
                    tokens.push(Token::LBrace);
                }
                '}' => {
                    self.next_char();
                    tokens.push(Token::RBrace);
                }
                ';' => {
                    self.next_char();
                    tokens.push(Token::Semicolon);
                }
                '=' => {
                    if self.starts_with("==") {
                        self.pos += 2;
                        tokens.push(Token::EqEq);
                    } else {
                        self.next_char();
                    tokens.push(Token::Eq);
                    }
                }    
                '!' => {
                    if self.starts_with("!=") {
                        self.pos += 2;
                        tokens.push(Token::NotEq);
                    } else {
                        self.next_char();
                    tokens.push(Token::Error);
                    }
                }    
                '<' => {
                    if self.starts_with("<=") {
                        self.pos += 2;
                        tokens.push(Token::Le);
                    } else {
                        self.next_char();
                        tokens.push(Token::Lt);
                    }
                }        
                '>' => {
                    if self.starts_with(">=") {
                        self.pos += 2;
                        tokens.push(Token::Ge);
                    } else {
                        self.next_char();
                        tokens.push(Token::Error);
                    }
                }       
                '*' => {
                    self.next_char();
                    tokens.push(Token::Star);
                }
                '/' => {
                    self.next_char();
                    tokens.push(Token::Slash);
                }
                
                
                
                c if c.is_ascii_digit() => {
                    let mut num = String::new();
                    while let Some(d) = self.peek_char() {
                        if d.is_ascii_digit() {
                            num.push(d);
                            self.next_char();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Number(num.parse().unwrap()));
                }
                c if c.is_alphabetic() || c == '_' => {
                    let mut ident = String::new();
                    ident.push(c);
                    self.next_char();
                    while let Some(nc) = self.peek_char() {
                        if nc.is_alphanumeric() || nc == '_' {
                            ident.push(nc);
                            self.next_char();

                        } else {
                            break;
                        }
                    }
                    match ident.as_str() {
                        "fn" => tokens.push(Token::Fn),
                        "let" =>tokens.push(Token::Let),
                        "if" => tokens.push(Token::If),
                        "else" => tokens.push(Token::Else),
                        "while" => tokens.push(Token::While),
                        _ => tokens.push(Token::Ident(ident)),
                    }
                }
                _ => {
                    self.next_char();
                    tokens.push(Token::Error);
                }
            }
        }

        tokens
    }
}
