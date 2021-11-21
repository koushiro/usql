use usql_core::{ansi::AnsiKeyword, CustomDialect, Dialect, DialectLexerConf, DialectParserConf};
use usql_lexer::{Lexer, LexerError};

pub type MyDialect1 = CustomDialect<AnsiKeyword, MyDialect1LexerConfig, MyDialectParserConfig>;
pub type MyDialect2 = CustomDialect<AnsiKeyword, MyDialect2LexerConfig, MyDialectParserConfig>;

#[derive(Clone, Debug, Default)]
pub struct MyDialect3 {
    lexer_conf: MyDialect3LexerConfig,
    parser_conf: MyDialectParserConfig,
}

impl Dialect for MyDialect3 {
    type Keyword = AnsiKeyword;
    type LexerConf = MyDialect3LexerConfig;
    type ParserConf = MyDialectParserConfig;

    fn lexer_conf(&self) -> &Self::LexerConf {
        &self.lexer_conf
    }

    fn parser_conf(&self) -> &Self::ParserConf {
        &self.parser_conf
    }
}

#[derive(Clone, Debug, Default)]
pub struct MyDialect1LexerConfig;
impl DialectLexerConf for MyDialect1LexerConfig {}

#[derive(Clone, Debug, Default)]
pub struct MyDialect2LexerConfig;
impl DialectLexerConf for MyDialect2LexerConfig {
    fn ignore_comment(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug, Default)]
pub struct MyDialect3LexerConfig;
impl DialectLexerConf for MyDialect3LexerConfig {
    fn ignore_whitespace(&self) -> bool {
        false
    }

    fn ignore_comment(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug, Default)]
pub struct MyDialectParserConfig;
impl DialectParserConf for MyDialectParserConfig {}

fn main() -> Result<(), LexerError> {
    let input = "\
    --this is single line comment\n\
    SELECT * FROM a WHERE id = 1\
    ";

    let dialect = MyDialect1::default();
    let mut lexer = Lexer::new(&dialect, input);
    let tokens = lexer.tokenize()?;
    println!("MyDialect1: {:#?}", tokens);

    let dialect = MyDialect2::default();
    let mut lexer = Lexer::new(&dialect, input);
    let tokens = lexer.tokenize()?;
    println!("MyDialect2: {:#?}", tokens);

    let dialect = MyDialect3::default();
    let mut lexer = Lexer::new(&dialect, input);
    let tokens = lexer.tokenize()?;
    println!("MyDialect3: {:#?}", tokens);

    Ok(())
}
