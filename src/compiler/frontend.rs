use peg::parser;

#[derive(Debug, Clone)]
pub enum Token {

    NoOp,

    Comment(String),
    Assert(Box<Token>),
    Import(String),
    Print(Box<Token>),

    Function(String, Vec<Token>, Vec<Token>),
    AnonFunction(Vec<Token>, Vec<Token>),
    Class(Box<Token>, Vec<Token>),
    ClassMethodCall(String, String, Vec<Token>),
    Chain(Box<Token>, Vec<Token>),
    Identifier(String),

    Null,
    Integer(i32),
    Float(f32),
    Bool(bool),
    String(String),
    Variable(String, Box<Token>),
    Assign(String, Box<Token>),
    Array(Vec<Token>),
    Object(Box<Token>, Vec<Token>),

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
    ForI(Box<Token>, Box<Token>, Box<Token>, Vec<Token>),

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

    // top level rule
    pub rule script() -> Vec<Token>
        = WHITESPACE() f:(import() / class() / function() / comment())* WHITESPACE() { f }

    // statements
    rule statements() -> Vec<Token>
        = s:((single_statement() / control_flow())*) { s }

    // single statements followed by a semicolon
    rule single_statement() -> Token
        = WHITESPACE() s:(
            assert() /
            var() /
            assignment() /
            print() /
            call() /
            rtn() /
            chain()
        ) WHITESPACE() SEMICOLON()+ WHITESPACE() { s }  / expected!("single statement")

    // control flow statements without semicolon
    rule control_flow() -> Token
        = WHITESPACE() c:(
            if_else() /
            while_loop() /
            foreach_loop() /
            fori_loop()
        ) WHITESPACE() { c } / expected!("control flow")

    // import external file
    rule import() -> Token
        = "import" _ s:string() { Token::Import(s) }

    // single line comment
    rule comment() -> Token
        = "//" s:$([' ' |'a'..='z' | 'A'..='Z' | '0'..='9']*) NEWLINE() { Token::Comment(s.to_owned()) }

    // class definition
    rule class() -> Token
        = "class" WHITESPACE() i:identifier() WHITESPACE() "{" WHITESPACE()
        items:(WHITESPACE() item:(var_statement() / function()) WHITESPACE() { item })*
        WHITESPACE() "}" WHITESPACE()
    { Token::Class(Box::new(i), items) }

    // class member call chain
    rule chain() -> Token
        = o:chain_item() "." chain:((e:chain_item() {e}) ** ".") { Token::Chain(Box::new(o), chain) }

    rule chain_item() -> Token
        = item:(array_index() / call() / identifier()) { item }

    // function definition with parameters
    rule function() -> Token
        = "function" _ name:identifier() _ "()" stmts:block() WHITESPACE() { Token::Function(name.to_string(), vec![], stmts) }
        / "function" _ name:identifier() _ "(" params:param_list() ")" stmts:block() WHITESPACE() { Token::Function(name.to_string(), params, stmts) }

    // function call with arguments
    rule call() -> Token
        = i:identifier() "(" args:arg_list() ")" { Token::Call(Box::new(i), args) }

    // code block wrapped in curly brackets
    rule block() -> Vec<Token>
        = WHITESPACE() "{" WHITESPACE() stmts:statements() WHITESPACE() "}" { stmts }

    // assert expression
    rule assert() -> Token
        = "assert" _ e:expression() { Token::Assert(Box::new(e)) }

    // print value
    rule print() -> Token
        = "print" _ e:expression() { Token::Print(Box::new(e)) }

    // anonymous function call
    rule anonfunc() -> Token
        = "function(" params:param_list() ")" stmts:block()
        { Token::AnonFunction(params, stmts) }

    // single var statement with a semicolon at the end
    rule var_statement() -> Token
        = WHITESPACE() v:var() WHITESPACE() SEMICOLON()+ WHITESPACE() { v }

    // variable declaration either with a value or default to null
    rule var() -> Token
        = "var" _ name:identifier() WHITESPACE() "=" WHITESPACE() e:expression() {  Token::Variable(name.to_string(), Box::new(e)) } /
          "var" _ name:identifier() { Token::Variable(name.to_string(), Box::new(Token::Null)) }




    // existing variable assignment
    rule assignment() -> Token
        = name:assignment_left() WHITESPACE() "=" WHITESPACE() r:expression() {  Token::Assign(name.to_string(), Box::new(r)) }
        / expected!("variable assignment")

    rule assignment_left() -> Token
        = o:assignment_left_item() "." chain:((e:assignment_left_item() {e}) ** ".") { Token::Chain(Box::new(o), chain) }
        / a:assignment_left_item() { a }

    rule assignment_left_item() -> Token
        = item:(array_index() / identifier()) { item }


    rule if_else() -> Token
        = "if" _ e:expression() WHITESPACE() "{" WHITESPACE() then_body:statements() WHITESPACE() "}" WHITESPACE()
            else_body:("else" _ "{" WHITESPACE() s:statements() WHITESPACE() "}" { s })?
        { Token::IfElse(Box::new(e), then_body, else_body) }

    rule while_loop() -> Token
        = "while" _ e:evaluation() s:block()
        { Token::WhileLoop(Box::new(e), s) }

    rule evaluation() -> Token
        = "(" e:expression() ")" { e } / e:expression() { e }

    rule foreach_loop() -> Token
        = "for" _ "(" _ i:identifier() _ "in" _ e:( identifier() / list()) _ ")" s:block()
        { Token::ForEach(Box::new(i), Box::new(e), s) }

    rule fori_loop() -> Token
        = "for" _ "(" _ v:(var() / assignment()) _ ";" _ to:expression() _ ";" _ step:assignment() _ ")" s:block()
        { Token::ForI(Box::new(v), Box::new(to), Box::new(step), s) }


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
    }

    rule literal() -> Token
        = f:float() { Token::Float(f) }
        / i:integer() { Token::Integer(i) }
        / c:anonfunc() { c }
        / a:array_index() { a }
        / c:chain() { c }
        / new_object_call()
        / i:identifier() { i }
        / n:null() { n }
        / b:boolean() { b }
        / "\"" s:string() "\"" { Token::String(s) }
        / list()
        / json()

    rule null() -> Token
        = "null" { Token::Null }

    rule boolean() -> Token
        = "true" { Token::Bool(true) }
        / "false" { Token::Bool(false) }

    rule new_object_call() -> Token
        = quiet!{"new" _ i:identifier() "(" args:arg_list() ")" { Token::Object(Box::new(i), args) } }

    rule arg_list() -> Vec<Token>
        = quiet!{args:((_ e:expression() _ {e}) ** ",") { args } }

    rule param_list() -> Vec<Token>
        = quiet!{args:((_ e:identifier() _ {e}) ** ",") { args } }

    // identifier starts with a letter or underscore, followed by any number of letters, numbers, or underscores, returns a string
    rule identifier_as_string() -> String
        = n:$(['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) { n.to_owned() }

    rule identifier() -> Token
        = n:$(['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) { Token::Identifier(n.to_owned()) }
        / expected!("identifier")

    rule array_index() -> Token
        = i:identifier() indexes:("[" e:expression() "]" { e })+ { Token::Index(Box::new(i), indexes) }
        / expected!("array index")

    rule string() -> String
        = n:$([^'"']*) { n.to_owned() }
        / expected!("string")

    rule integer() -> i32
        = n:$("-"? ['0'..='9']+) { n.parse().unwrap() }

    rule float() -> f32
        = n:$("-"? ['0'..='9']+ "." ['0'..='9']+) { n.parse().unwrap() }

    rule list() -> Token
        = quiet!{ "[" WHITESPACE() elements:(( WHITESPACE() e:expression() _ {e}) ** ",") WHITESPACE() "]" { Token::Array(elements) } }

    rule json() -> Token
        = quiet!{ "{" WHITESPACE() kv:(( WHITESPACE() "\"" k:string() "\"" _ ":" _ e:expression() _ {  Token::KeyValuePair(k, Box::new(e)) } ) ** ",") WHITESPACE() "}" { Token::Dictionary(kv) } }



    // statement ends with at least one semicolon
    rule SEMICOLON() = quiet!{";"}

    rule _() =  quiet!{[' ' | '\t']*}
    rule NEWLINE() = quiet!{ ['\n'|'\r'] }
    rule NEWLINES() = quiet!{ ['\n'|'\r']* }
    rule WHITESPACE() = quiet!{ [' '|'\t'|'\n'|'\r']* }
    rule UTF8CHAR() -> char = quiet!{ c:([^ '\x00'..='\x1F' | '\t' | '\n'|'\r']) { c } }

});
