use usql_core::{ansi::AnsiKeyword, CustomDialect, Dialect, DialectLexerConf, DialectParserConf};
use usql_lexer::{Lexer, LexerError};

pub type MyDialect1 = CustomDialect<AnsiKeyword, MyDialect1LexerConfig, MyDialectParserConfig>;

#[derive(Clone, Debug, Default)]
pub struct MyDialect2 {
    lexer_conf: MyDialect2LexerConfig,
    parser_conf: MyDialectParserConfig,
}

impl Dialect for MyDialect2 {
    type Keyword = AnsiKeyword;
    type LexerConf = MyDialect2LexerConfig;
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
impl DialectLexerConf for MyDialect1LexerConfig {
    fn ignore_whitespace(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug, Default)]
pub struct MyDialect2LexerConfig;
impl DialectLexerConf for MyDialect2LexerConfig {
    fn ignore_whitespace(&self) -> bool {
        true
    }

    fn ignore_comment(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug, Default)]
pub struct MyDialectParserConfig;
impl DialectParserConf for MyDialectParserConfig {}

fn main() -> Result<(), LexerError> {
    let input = r#"
        --this is single line comment
        SELECT * FROM a WHERE id = 1
    "#;

    let dialect = MyDialect1::default();
    let tokens = Lexer::new(&dialect, input).tokenize()?;
    println!("MyDialect1: {:#?}", tokens);

    let dialect = MyDialect2::default();
    let tokens = Lexer::new(&dialect, input).tokenize()?;
    println!("MyDialect2: {:#?}", tokens);

    Ok(())
}
