use crate::keywords::*;

#[derive(Copy, Clone, Debug)]
pub struct LexerConfig<K = AnsiKeyword> {
    pub(crate) keyword: K,
    pub(crate) max_multi_line_comment: u32,
}

impl<K: KeywordDef> Default for LexerConfig<K> {
    fn default() -> Self {
        Self {
            keyword: K,
            max_multi_line_comment: 128,
        }
    }
}

impl<K: KeywordDef> LexerConfig<K> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> LexerConfigBuilder<K> {
        LexerConfigBuilder::<K>::default()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LexerConfigBuilder<K = AnsiKeyword> {
    keyword: K,
    max_multi_line_comment: u32,
}

impl<K: KeywordDef> Default for LexerConfigBuilder<K> {
    fn default() -> Self {
        Self {
            keyword: K,
            max_multi_line_comment: 128,
        }
    }
}

impl<K: KeywordDef> LexerConfigBuilder<K> {
    ///
    pub fn max_multi_line_comment(&mut self, max: u32) -> &mut Self {
        self.max_multi_line_comment = max;
        self
    }

    ///
    pub fn build(self) -> LexerConfig<K> {
        LexerConfig {
            keyword: self.keyword,
            max_multi_line_comment: self.max_multi_line_comment,
        }
    }
}

