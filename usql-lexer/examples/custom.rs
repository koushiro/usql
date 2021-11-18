use usql_core::{
    ansi::AnsiKeyword, CustomDialect, Dialect, DialectLexerConf,
    DialectParserConf,
};
use usql_lexer::{Lexer, LexerError};

pub type MyDialect1 = CustomDialect<AnsiKeyword, MyDialectLexerConfig, MyDialectParserConfig>;

#[derive(Clone, Debug, Default)]
pub struct MyDialect2 {
    lexer_conf: MyDialectLexerConfig,
    parser_conf: MyDialectParserConfig,
}

impl Dialect for MyDialect2 {
    type Keyword = AnsiKeyword;
    type LexerConf = MyDialectLexerConfig;
    type ParserConf = MyDialectParserConfig;

    fn lexer_conf(&self) -> &Self::LexerConf {
        &self.lexer_conf
    }

    fn parser_conf(&self) -> &Self::ParserConf {
        &self.parser_conf
    }
}

#[derive(Clone, Debug, Default)]
pub struct MyDialectLexerConfig;
impl DialectLexerConf for MyDialectLexerConfig {}

#[derive(Clone, Debug, Default)]
pub struct MyDialectParserConfig;
impl DialectParserConf for MyDialectParserConfig {}

fn main() -> Result<(), LexerError> {
    let input = "SELECT * FROM a WHERE id = 1";

    let mut lexer = Lexer::new(input, MyDialect1::default());
    let tokens = lexer.tokenize()?;
    println!("MyDialect1: {:#?}", tokens);

    let mut lexer = Lexer::new(input, MyDialect2::default());
    let tokens = lexer.tokenize()?;
    println!("MyDialect2: {:#?}", tokens);

    Ok(())
}
