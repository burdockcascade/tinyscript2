#[derive(Debug, Clone)]
pub enum Token {

    Comment(String),
    Assert(Box<Token>),
    Import(String),
    Print(Box<Token>),

    Function(String, Vec<Token>, Vec<Token>),
    
    AnonFunction(Vec<Token>, Vec<Token>),
    Class(String, Vec<Token>),
    Constructor(Vec<Token>, Vec<Token>),
    Chain(Box<Token>, Vec<Token>),
    Identifier(String),

    Variable(Box<Token>, Box<Token>),
    Assign(Box<Token>, Box<Token>),

    Null,
    Integer(i32),
    Float(f32),
    Bool(bool),
    String(String),
    Array(Vec<Token>),
    Object(Box<Token>, Vec<Token>),

    Dictionary(Vec<Token>),
    KeyValuePair(String, Box<Token>),

    ArrayIndex(Box<Token>, Box<Token>),

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
            Token::Identifier(name) => name.to_string(),
            Token::String(s) => s.to_string(),
            _ => String::from("")
        }
    }
}