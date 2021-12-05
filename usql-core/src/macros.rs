// Modified based on the https://github.com/sqlparser-rs/sqlparser-rs/blob/main/src/keywords.rs

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
/// and defines an ALL_KEYWORDS array of the defined constants.
///
/// **NOTE**: All keywords should be sorted to be able to match using binary search.
macro_rules! define_all_keywords {
    (
        $(
            $keyword:ident $(= $string_keyword:expr)?
        ),*
    ) => {
        /// All keywords
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub enum Keyword {
            $($keyword),*
        }

        impl ::core::fmt::Display for Keyword {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Debug::fmt(self, f)
            }
        }
    }
}

/// Define a list of keywords of the dialect.
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
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $name;

        mod __private {
            use super::$name;

            $( $crate::kw_def!($keyword $(= $string_keyword)?); )*

            impl $crate::KeywordDef for $name {
                const KEYWORDS: &'static [$crate::Keyword] = &[
                    $($crate::Keyword::$keyword),*
                ];

                const KEYWORDS_STRING: &'static [&'static str] = &[
                    $($keyword),*
                ];

                const RESERVED_KEYWORDS: &'static [$crate::Keyword] = &[
                    $($crate::Keyword::$keyword),*
                ];
            }
        }
    };

    (
        $(#[$doc:meta])*
        $name:ident => {
            $(
                $keyword:ident $(= $string_keyword:expr)?
            ),*
        };
        $reserved:ident => {
            $( $reserved_keyword:ident ),*
        }
    ) => {
        $(#[$doc])*
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $name;

        const _: () = {
            struct $reserved;
        };

        mod __private {
            use super::$name;

            $( $crate::kw_def!($keyword $(= $string_keyword)?); )*

            impl $crate::KeywordDef for $name {
                const KEYWORDS: &'static [$crate::Keyword] = &[
                    $($crate::Keyword::$keyword),*
                ];

                const KEYWORDS_STRING: &'static [&'static str] = &[
                    $($keyword),*
                ];

                const RESERVED_KEYWORDS: &'static [$crate::Keyword] = &[
                    $($crate::Keyword::$reserved_keyword),*
                ];
            }
        }
    }
}
