/// Defines a string constant for a single keyword: `kw_def!(SELECT);`,
/// which expands to `const SELECT: &'static str = "SELECT";`
#[macro_export]
macro_rules! kw_def {
    ($ident:ident = $string_keyword:expr) => {
        const $ident: &'static str = $string_keyword;
    };
    ($ident:ident) => {
        $crate::kw_def!($ident = stringify!($ident));
    };
}

/// Expands to a list of `kw_def!()` invocations for each keyword
/// and implement the `Display` and the `KeywordDef` traits for the list.
///
/// **NOTE**: All keywords should be sorted to be able to match using binary search.
#[macro_export]
macro_rules! define_keyword {
    (
        $(#[$doc:meta])*
        $name:ident => {
            $(
                $keyword:ident $(= $string_keyword:expr)?
            ),*
        }
    ) => {
        $(#[$doc])*
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub enum $name {
            $($keyword),*
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Debug::fmt(self, f)
            }
        }

        $( $crate::kw_def!($keyword $(= $string_keyword)?); )*

        impl $crate::KeywordDef for $name {
            const KEYWORDS: &'static [Self] = &[
                $(Self::$keyword),*
            ];
            const KEYWORD_STRINGS: &'static [&'static str] = &[
                $($keyword),*
            ];
        }
    }
}
