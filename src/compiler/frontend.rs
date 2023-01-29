use peg::parser;

#[derive(Debug, Clone)]
pub enum Token {

    Assert(Box<Token>),
    Import(String),
    Print(Box<Token>),

    Function(Box<Token>, Vec<Token>, Vec<Token>),
    AnonFunction(Vec<Token>, Vec<Token>),
    Class(Box<Token>, Vec<Token>),
    Identifier(String),

    Null,
    Integer(i32),
    Float(f32),
    Bool(bool),
    String(String),
    Variable(Box<Token>, Box<Token>),
    Assign(Box<Token>, Box<Token>),
    Array(Vec<Token>),

    Dictionary(Vec<Token>),
    KeyValuePair(String, Box<Token>),

    Index(Box<Token>, Vec<Token>),
    IndexAssign(Box<Token>, Vec<Token>, Box<Token>),

    Eq(Box<Token>, Box<Token>),
    Ne(Box<Token>, Box<Token>),
    Lt(Box<Token>, Box<Token>),
    Le(Box<Token>, Box<Token>),
    Gt(Box<Token>, Box<Token>),
    Ge(Box<Token>, Box<Token>),
    Add(Box<Token>, Box<Token>),
    Sub(Box<Token>, Box<Token>),
    Mul(Box<Token>, Box<Token>),
    Div(Box<Token>, Box<Token>),
    Pow(Box<Token>, Box<Token>),

    IfElse(Box<Token>, Vec<Token>, Option<Vec<Token>>),
    WhileLoop(Box<Token>, Vec<Token>),
    ForEach(Box<Token>, Box<Token>, Vec<Token>),
    ForI(Box<Token>, Box<Token>, Box<Token>, Option<Box<Token>>, Vec<Token>),

    Call(Box<Token>, Vec<Token>),
    Return(Box<Token>)
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Token::Function(name, _, _) => name.to_string(),
            Token::Identifier(name) => String::from(name),
            _ => String::from("")
        }
    }
}

parser!(pub grammar parser() for str {

    pub rule script() -> Vec<Token>
        = WHITESPACE() f:(
            function()*
        ) WHITESPACE() { f }

    rule statements() -> Vec<Token>
        = s:(statement()*) { s }

    rule statement() -> Token
        = WHITESPACE() e:(
            import() /
            class() /
            var() /
            print() /
            assignment() /
            index_assigment() /
            assert() /
            call() /
            function() /
            if_else() /
            while_loop() /
            foreach_loop() /
            fori_loop() /
            rtn()
        ) WHITESPACE() { e }

    rule import() -> Token
        = "import" _ s:string() _ NEWLINE() { Token::Import(s) }

    rule assert() -> Token
        = "assert" _ e:expression() WHITESPACE() { Token::Assert(Box::new(e)) }

    rule class() -> Token
        = "class" _ i:identifier() WHITESPACE()
        items:(WHITESPACE() t:(var() / function()) WHITESPACE() { t })*
        "end"
    { Token::Class(Box::new(i), items) }

    rule print() -> Token
        = "print" _ e:expression() WHITESPACE() { Token::Print(Box::new(e)) }

    rule function() -> Token
        = "function" _ name:identifier() _ "(" params:((_ i:identifier() _ {i}) ** ",") ")" WHITESPACE()
            stmts:statements() WHITESPACE()
            "end" WHITESPACE()
        { Token::Function(Box::new(name), params, stmts) }

    rule anonfunc() -> Token
        = "function(" params:((_ i:identifier() _ {i}) ** ",") ")" WHITESPACE()
            stmts:statements() WHITESPACE()
            "end" WHITESPACE()
        { Token::AnonFunction(params, stmts) }

    rule call() -> Token
        = i:identifier() "(" args:((_ e:expression() _ {e}) ** ",") ")" { Token::Call(Box::new(i), args) }

    rule var() -> Token
        = "var" _ i:identifier() _ "=" _ e:expression() {  Token::Variable(Box::new(i), Box::new(e)) }

    rule assignment() -> Token
        = i:identifier() _ "=" _ e:expression() {  Token::Assign(Box::new(i), Box::new(e)) }

    rule index_assigment() -> Token
        = i:identifier() indexes:("[" e:expression() "]" { e })+  _ "=" _ e:expression()  { Token::IndexAssign(Box::new(i), indexes, Box::new(e)) }

    rule if_else() -> Token
        = "if" _ e:expression() WHITESPACE() "{" WHITESPACE() then_body:statements() WHITESPACE() "}" WHITESPACE()
            else_body:("else" _ "{" WHITESPACE() s:statements() WHITESPACE() "}" { s })?
        { Token::IfElse(Box::new(e), then_body, else_body) }

    rule while_loop() -> Token
        = "while" _ e:expression() WHITESPACE() "do" WHITESPACE()
            s:statements() WHITESPACE()
            "end"
        { Token::WhileLoop(Box::new(e), s) }

    rule foreach_loop() -> Token
        = "for" _ i:identifier() _ "in" _ e:( identifier() / list()) WHITESPACE() "do" WHITESPACE()
            s:statements() WHITESPACE()
            "end"
        { Token::ForEach(Box::new(i), Box::new(e), s) }

    rule fori_loop() -> Token
        = "for" _ i:identifier() _ "=" _ from:expression() _ "to" _ to:expression() _ step:("step" _ e:expression() { Box::new(e) } )? _ "do" WHITESPACE()
            s:statements() WHITESPACE()
            "end"
        { Token::ForI(Box::new(i), Box::new(from), Box::new(to), step, s) }

    rule rtn() -> Token
        = "return" _ e:expression() { Token::Return(Box::new(e)) }

    rule expression() -> Token = precedence!{
        a:@ _ "==" _ b:(@) { Token::Eq(Box::new(a), Box::new(b)) }
        a:@ _ "!=" _ b:(@) { Token::Ne(Box::new(a), Box::new(b)) }
        a:@ _ "<"  _ b:(@) { Token::Lt(Box::new(a), Box::new(b)) }
        a:@ _ "<=" _ b:(@) { Token::Le(Box::new(a), Box::new(b)) }
        a:@ _ ">"  _ b:(@) { Token::Gt(Box::new(a), Box::new(b)) }
        a:@ _ ">=" _ b:(@) { Token::Ge(Box::new(a), Box::new(b)) }
        --
        a:@ _ "+" _ b:(@) { Token::Add(Box::new(a), Box::new(b)) }
        a:@ _ "-" _ b:(@) { Token::Sub(Box::new(a), Box::new(b)) }
        --
        a:@ _ "*" _ b:(@) { Token::Mul(Box::new(a), Box::new(b)) }
        a:@ _ "/" _ b:(@) { Token::Div(Box::new(a), Box::new(b)) }
        a:@ _ "^" _ b:(@) { Token::Pow(Box::new(a), Box::new(b)) }
        --
        l:literal() { l }
        i:identifier() "(" args:((_ e:expression() _ {e}) ** ",") ")" { Token::Call(Box::new(i), args) }
        a:array_index() { a }
        i:identifier() { i }
    }

    rule func_call() -> Token
        = quiet!{i:identifier() "(" args:((_ e:expression() _ {e}) ** ",") ")" { Token::Call(Box::new(i), args) } }

    rule identifier() -> Token
        = quiet!{ n:$(['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) { Token::Identifier(n.to_owned()) } }
        / expected!("identifier")

    rule array_index() -> Token
        = i:identifier() indexes:("[" e:expression() "]" { e })+ { Token::Index(Box::new(i), indexes) }

    rule string() -> String
        = quiet!{ n:$([^'"']*) { n.to_owned() } }
        / expected!("string")

    rule integer() -> i32
        = quiet!{ n:$("-"? ['0'..='9']+) { n.parse().unwrap() } }

    rule list() -> Token
        = quiet!{ "[" WHITESPACE() elements:(( WHITESPACE() e:expression() _ {e}) ** ",") WHITESPACE() "]" { Token::Array(elements) } }

    rule json() -> Token
        = quiet!{ "{" WHITESPACE() kv:(( WHITESPACE() "\"" k:string() "\"" _ ":" _ e:expression() _ {  Token::KeyValuePair(k, Box::new(e)) } ) ** ",") WHITESPACE() "}" { Token::Dictionary(kv) } }

    rule literal() -> Token
        = n:$(['0'..='9']+ "." ['0'..='9']+) { Token::Float(n.parse().unwrap()) }
        / i:integer() { Token::Integer(i) }
        / "null" { Token::Null }
        / "true" { Token::Bool(true) }
        / "false" { Token::Bool(false) }
        / "\"" s:string() "\"" { Token::String(s) }
        / list()
        / json()
        / c:anonfunc() { c }

    rule _() =  quiet!{[' ' | '\t']*}
    rule NEWLINE() = quiet!{ ['\n'|'\r'] }
    rule NEWLINES() = quiet!{ ['\n'|'\r']* }
    rule WHITESPACE() = quiet!{ [' '|'\t'|'\n'|'\r']* }

});
