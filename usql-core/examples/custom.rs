use usql_core::{define_keyword, CustomDialect, DialectLexerConf, DialectParserConf};

define_keyword! {
    MyKeyword => {
        ABORT,
        ABS,
        ACCESSIBLE
    }
}

pub type MyDialect = CustomDialect<MyKeyword, MyDialectLexerConfig, MyDialectParserConfig>;

#[derive(Clone, Debug, Default)]
pub struct MyDialectLexerConfig;
impl DialectLexerConf for MyDialectLexerConfig {}

#[derive(Clone, Debug, Default)]
pub struct MyDialectParserConfig;
impl DialectParserConf for MyDialectParserConfig {}

fn main() {
    let _dialect = MyDialect::default();
}
