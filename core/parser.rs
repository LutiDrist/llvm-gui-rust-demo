use crate::ast::{Expr, Stmt, Function};
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: Token) -> Option<()> {
        match self.next()? {
            t if t == expected => Some(()),
            _ => None,
        }
    }

    fn lookahead_is_eq(&self) -> bool {
        if self.pos + 1 >= self.tokens.len() {return false;}
        matches!(self.tokens[self.pos + 1], Token::Eq)
    }


    pub fn parse_function(&mut self) -> Option<Function> {
        self.expect(Token::Fn)?;
        let name = if let Token::Ident(n) = self.next()? { n } else { return None };
        self.expect(Token::LParen)?;
        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;
        let  body = self.parse_block_stmts()?;
        Some(Function { name, body })
    }

    fn parse_block_stmts(&mut self) -> Option<Vec<Stmt>> {
        let mut out = Vec::new();
        loop {
            match self.peek()? {
                Token::RBrace => {
                    self.next(); break;
                }
                _ => {
                    let st = self.parse_stmt()?;
                    out.push(st);
                }
            }
        }
        Some(out)
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.peek()? {
            Token::Let => {
                self.next(); // consume `let`
                let name = if let Token::Ident(n) = self.next()? { n } else { return None;};
                self.expect(Token::Eq)?;
                let expr =  self.parse_expr()?;
                self.expect(Token::Semicolon)?;
            Some(Stmt::Let(name, expr))
        }
        Token::If => {
            self.next();
            self.expect(Token::LParen)?;
            let cond = self.parse_expr()?;
            self.expect(Token::RParen)?;
            self.expect(Token::LBrace)?;
            let then_body = self.parse_block_stmts()?;

            let else_body = if matches!(self.peek(), Some(Token::Else)) {
                self.next();
                self.expect(Token::LBrace)?;
                Some(self.parse_block_stmts()?)
            } else {None };
            Some(Stmt::If { cond, then_body, else_body})
        }
        Token::While => {
            self.next();
            self.expect(Token::LParen)?;
            let cond = self.parse_expr()?;
            self.expect(Token::RParen)?;
            self.expect(Token::LBrace)?;
            let body = self.parse_block_stmts()?;
            Some(Stmt::While { cond, body })
        }
        Token::Ident(name) if self.lookahead_is_eq() => {
            let _name = name.clone();
            self.next();
            self.expect(Token::Eq)?;
            let expr = self.parse_expr()?;
            self.expect(Token::Semicolon)?;
            Some(Stmt::Expr(expr))
        } 
        _ => {
            let expr = self.parse_expr()?;
            self.expect(Token::Semicolon)?;
            Some(Stmt::Expr(expr))
            }
        }
    }
    fn parse_expr(&mut self) -> Option<Expr> { self.parse_cmp()}

    fn parse_cmp(&mut self) -> Option<Expr> {
        let mut left = self.parse_add()?;
        loop {
            let op = match self.peek() {
                Some(Token::EqEq) => "==",
                Some(Token::NotEq) => "!=",
                Some(Token::Lt) => "<",
                Some(Token::Gt) => ">",
                Some(Token::Le) => "<=",
                Some(Token::Ge) => ">=",
                _=> break
            }.to_string();
            self.next();
            let rignt = self.parse_add()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(rignt));
        }
        Some(left)
    }

    fn parse_add(&mut self) -> Option<Expr> {
        let mut left =  self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => "+",
                Some(Token::EqEq) => "-",
                _ => break,
            }.to_string();
            self.next();
            let right = self.parse_mul()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Some(left)
    }
    fn parse_mul(&mut self) -> Option<Expr> {
        let mut left = self.parse_primary()?;
        loop {
                        let op = match self.peek() {
                Some(Token::Star) => "*",
                Some(Token::Slash) => "/",
                _ => break,
            }.to_string();
            self.next();
            let right = self.parse_primary()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
               Some(left)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        match self.next()? {
            Token::Number(n) => Some(Expr::Number(n)),
            Token::Ident(name) => Some(Expr::Ident(name)),
            Token::LParen => {
                let e = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Some(e)
            }
            _ => None,
        }
    }
}
