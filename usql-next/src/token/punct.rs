define_punctuation! {
    "."     => pub struct Dot/1
    ","     => pub struct Comma/1
    ";"     => pub struct Semicolon/1
    ":"     => pub struct Colon/1
    "::"    => pub struct DoubleColon/2

    "("     => pub struct LeftParen/1
    ")"     => pub struct RightParen/1
    "["     => pub struct LeftBracket/1
    "]"     => pub struct RightBracket/1
    "{"     => pub struct LeftBrace/1
    "}"     => pub struct RightBrace/1

    "="     => pub struct Eq/1
    "<>"    => pub struct NotEq/2
    "<"     => pub struct Less/1
    "<="    => pub struct LessEq/2
    ">"     => pub struct Greater/1
    ">="    => pub struct GreaterEq/2

    "<<"    => pub struct LeftShift/2
    ">>"    => pub struct RightShift/2

    "+"     => pub struct Plus/1
    "-"     => pub struct Minus/1
    "*"     => pub struct Asterisk/1
    "/"     => pub struct Slash/1
    "%"     => pub struct Percent/1

    "^"     => pub struct Caret/1
    "!"     => pub struct Exclamation/1
    "?"     => pub struct Question/1
    "~"     => pub struct Tilde/1
    "&"     => pub struct Ampersand/1
    "|"     => pub struct Pipe/1
    "||"    => pub struct DoublePipe/2
    "\\"    => pub struct Backslash/1
    "#"     => pub struct Sharp/1
    "@"     => pub struct At/1
}
