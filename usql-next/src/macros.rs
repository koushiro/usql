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
macro_rules! define_keywords {
    (
        $(
            $keyword:ident $(= $string_keyword:expr)?
        )*
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
macro_rules! custom_keywords {
    (
        $(#[$doc:meta])*
        $name:ident => {
            $(
                $keyword:ident $(= $string_keyword:expr)?
            )*
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
            )*
        };
        $reserved:ident => {
            $( $reserved_keyword:ident )*
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

/*
macro_rules! define_keywords {
    (
        $($name:ident)*
    ) => {
        $(
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
            pub struct $name;

            impl ::core::fmt::Display for $name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    f.write_str(stringify!($name))
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! custom_keywords {
    () => {};
}
*/

// ================================================================================================

macro_rules! define_punctuation {
    (
        $($token:tt => pub struct $name:ident/$len:tt)*
    ) => {
        $(
            #[doc(hidden)]
            #[derive(Copy, Clone, Default)]
            pub struct $name {
                pub span: [$crate::Span; $len],
            }

            impl ::core::fmt::Debug for $name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    f.write_str(stringify!($token))
                }
            }

            impl ::core::fmt::Display for $name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    f.write_str(stringify!($token))
                }
            }

            impl PartialEq for $name {
                fn eq(&self, other: &Self) -> bool {
                    self.span == other.span
                }
            }

            impl ::core::cmp::Eq for $name {}

            impl ::core::hash::Hash for $name {
                fn hash<H: ::core::hash::Hasher>(&self, _state: &mut H) {}
            }
        )*
    };
}

/// Define a type that supports parsing and printing a multi-character symbol
/// as if it were a punctuation token.
///
/// # Usage
///
/// ```
/// usql_next::custom_punctuation!(CubeRoot, ||/);
/// ```
#[macro_export]
macro_rules! custom_punctuation {
    (
        $name:ident, $($tt:tt)+
    ) => {
        #[doc(hidden)]
        #[derive(Copy, Clone, Default)]
        pub struct $name {
            pub spans: $crate::custom_punctuation_repr!($($tt)+),
        }

        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.write_str(stringify!($token))
            }
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.write_str(stringify!($token))
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.span == other.span
            }
        }

        impl ::core::cmp::Eq for $name {}

        impl ::core::hash::Hash for $name {
            fn hash<H: ::core::hash::Hasher>(&self, _state: &mut H) {}
        }
    };
}

// Not public API
#[macro_export]
#[doc(hidden)]
macro_rules! custom_punctuation_repr {
    (
        $($tt:tt)+
    ) => {
        [$crate::Span; 0 $(+ $crate::custom_punctuation_len!(lenient, $tt))+]
    };
}

// Not public API
#[macro_export]
#[doc(hidden)]
#[rustfmt::skip]
macro_rules! custom_punctuation_len {
    ($mode:ident, .)    => { 1 };
    ($mode:ident, ,)    => { 1 };
    ($mode:ident, ;)    => { 1 };
    ($mode:ident, :)    => { 1 };
    ($mode:ident, ::)   => { 2 };

    ($mode:ident, <)    => { 1 };
    ($mode:ident, <=)   => { 2 };
    ($mode:ident, >)    => { 1 };
    ($mode:ident, >=)   => { 2 };

    ($mode:ident, <<)   => { 2 };
    ($mode:ident, >>)   => { 2 };

    ($mode:ident, +)    => { 1 };
    ($mode:ident, -)    => { 1 };
    ($mode:ident, *)    => { 1 };
    ($mode:ident, /)    => { 1 };
    ($mode:ident, %)    => { 1 };

    ($mode:ident, &)    => { 1 };
    ($mode:ident, |)    => { 1 };
    ($mode:ident, ^)    => { 1 };
    ($mode:ident, ~)    => { 1 };
    ($mode:ident, !)    => { 1 };
    ($mode:ident, ?)    => { 1 };
    ($mode:ident, |)    => { 1 };
    ($mode:ident, #)    => { 1 };
    ($mode:ident, @)    => { 1 };
    (lenient, $tt:tt)   => { 0 };
    (strict, $tt:tt)    => {{ $crate::custom_punctuation_unexpected!($tt); 0 }};
}

// Not public API.
#[doc(hidden)]
#[macro_export]
macro_rules! custom_punctuation_unexpected {
    () => {};
}
