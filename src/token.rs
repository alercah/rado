use failure::{format_err, Error, Fail};
use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Fail)]
#[fail(display = "{:?} is not a keyword", s)]
pub struct ParseKwError {
    s: String,
}

#[derive(Clone, Debug, Fail)]
#[fail(display = "{:?} is not a symbol", s)]
pub struct ParseSymError {
    s: String,
}

macro_rules! toks {
    {$name:ident; err $err:ident; all $all: ident; $($kw:ident <- $spl:expr),*,} => {
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub enum $name {
            $($kw),*,
        }

        impl FromStr for $name {
            type Err = $err;

            fn from_str(s: &str) -> Result<$name, $err> {
                match s {
                    $($spl => Ok($name::$kw)),*,
                    _ => Err($err{s: s.into()}),
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(match self {
                    $($name::$kw => $spl),*,
                })
            }
        }

        pub static $all: &'static [$name] = &[
            $($name::$kw),*,
        ];
    };
}

toks! { Kw;
    err ParseKwError;
    all ALL_KEYWORDS;
    // Declarations
    Region <- "region",
    Link <- "link",
    Item <- "item",
    Items <- "items",
    Location <- "location",
    Locations <- "locations",
    Fn <- "fn",
    Enum <- "enum",
    Config <- "config",
    Configset <- "configset",
    Random <- "random",
    If <- "if",
    Else <- "else",
    Modify <- "modify",
    Override <- "override",

    // Properties
    Requires <- "requires",
    Visible <- "visible",
    Unlock <- "unlock",
    Tag <- "tag",
    Alias <- "alias",
    Provides <- "provides",
    Progressive <- "progressive",
    Val <- "val",
    Max <- "max",
    Consumable <- "consumable",
    Restrict <- "restrict",
    Avail <- "avail",
    Infinity <- "infinity",
    Grants <- "grants",
    Count <- "count",
    Start <- "start",

    // Expressions & types not covered above
    Num <- "num",
    Bool <- "bool",
    Match <- "match",
    Min <- "min",

    // Prepositions
    With <- "with",
    To <- "to",
    From <- "from",
    In <- "in",
}

toks! { Sym;
    err ParseSymError;
    all ALL_SYMBOLS;
    // Delimeters
    LParen <- "(",
    RParen <- ")",
    LBrack <- "[",
    RBrack <- "]",
    LBrace <- "{",
    RBrace <- "}",

    // Punctuation
    Semi <- ";",
    Comma <- ",",
    Colon <- ":",
    Dot <- ".",
    Assign <- "=",
    Arrow <- "->",
    DoubleArrow <- "=>",

    // Operators
    Plus <- "+",
    Minus <- "-",
    Star <- "*",
    Slash <- "/",
    Percent <-"%",
    Eq <- "==",
    NEq <- "!=",
    LT <- "<",
    LE <- "<=",
    GT <- ">=",
    GE <- ">",
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Tok<'a> {
    Kw(Kw),
    Sym(Sym),
    Ident(Cow<'a, str>),
    Num(Cow<'a, str>, Option<Cow<'a, str>>),
    String(Cow<'a, str>),
}

impl<'a> fmt::Display for Tok<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tok::Kw(k) => write!(f, "{}", k),
            Tok::Sym(s) => write!(f, "{}", s),
            Tok::Ident(i) => write!(f, "{}", i),
            Tok::Num(w, None) => write!(f, "{}", w),
            Tok::Num(w, Some(d)) => write!(f, "{}.{}", w, d),
            Tok::String(s) => write!(f, "{:?}", s),
        }
    }
}

fn peek(s: &str) -> Option<char> {
    s.chars().next()
}

/// For a string starting on a block comment marker, advance up to the last
/// character of the block comment. It will recurse in order to handle nested
/// comments.
fn skip_block_comment(mut s: &str) -> Result<&str, Error> {
    assert!(s.len() >= 2);
    assert!(s.starts_with("/*"));
    s = &s[2..];

    // TODO: This feels kind of bad to search twice, but lazy_static/regex is a
    // lot of work for two two-character search patterns. Also this means every
    // recursion is checking for an EOF, which is pointless.
    loop {
        let end = s
            .find("*/")
            .ok_or(format_err!("unterminated block comment"))?;
        match s.find("/*") {
            Some(inner) if inner < end => {
                s = &skip_block_comment(&s[inner..])?[1..];
            }
            _ => break Ok(&s[end + 1..]),
        }
    }
}

pub fn lex<'a>(mut s: &'a str) -> Result<Vec<Tok<'a>>, Error> {
    let mut toks = Vec::new();
    while s.len() > 0 {
        let c = peek(s).unwrap();
        let rest = &s[1..];

        // Note that we unconditionally advance s at the end of the loop, so
        // multi-character tokens need to advance to their last character,
        // rather than past it.
        //
        // Throughout this code, we reborrow s by index past the first character
        // quite frequently. Non-ASCII characters might cause panics but they
        // are not legal, so it's fine for a first pass on the lexer.
        //
        // TODO: Mostly the reason for this is having to add -1 in a ton of
        // places so as to avoid having to advance manually in every branch.
        // Should probably think that through more.
        match c {
            '(' => toks.push(Tok::Sym(Sym::LParen)),
            ')' => toks.push(Tok::Sym(Sym::RParen)),
            '[' => toks.push(Tok::Sym(Sym::LBrack)),
            ']' => toks.push(Tok::Sym(Sym::RBrack)),
            '{' => toks.push(Tok::Sym(Sym::LBrace)),
            '}' => toks.push(Tok::Sym(Sym::RBrace)),
            ';' => toks.push(Tok::Sym(Sym::Semi)),
            ',' => toks.push(Tok::Sym(Sym::Comma)),
            ':' => toks.push(Tok::Sym(Sym::Colon)),
            '.' => toks.push(Tok::Sym(Sym::Dot)),
            '+' => toks.push(Tok::Sym(Sym::Plus)),
            '*' => toks.push(Tok::Sym(Sym::Star)),
            '%' => toks.push(Tok::Sym(Sym::Percent)),
            '/' => match peek(rest) {
                Some('/') => {
                    let i = s.find('\n').unwrap_or(s.len());
                    s = &s[i - 1..];
                }
                Some('*') => {
                    s = skip_block_comment(s)?;
                }
                _ => toks.push(Tok::Sym(Sym::Slash)),
            },
            '!' => if peek(rest) == Some('=') {
                s = &s[1..];
                toks.push(Tok::Sym(Sym::NEq));
            } else {
                return Err(format_err!("expected = after ! to make != token"));
            },
            '=' => match peek(rest) {
                Some('=') => {
                    s = &s[1..];
                    toks.push(Tok::Sym(Sym::Eq));
                }
                Some('>') => {
                    s = &s[1..];
                    toks.push(Tok::Sym(Sym::DoubleArrow));
                }
                _ => toks.push(Tok::Sym(Sym::Assign)),
            },
            '>' => if peek(rest) == Some('=') {
                s = &s[1..];
                toks.push(Tok::Sym(Sym::GE));
            } else {
                toks.push(Tok::Sym(Sym::GT));
            },
            '<' => if peek(rest) == Some('=') {
                s = &s[1..];
                toks.push(Tok::Sym(Sym::LE));
            } else {
                toks.push(Tok::Sym(Sym::LT));
            },
            '-' => if peek(rest) == Some('>') {
                s = &s[1..];
                toks.push(Tok::Sym(Sym::Arrow));
            } else {
                toks.push(Tok::Sym(Sym::Minus));
            },
            c if c.is_ascii_digit() => {
                let i = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
                let (w, mut f) = (s[0..i].into(), None);
                s = &s[i - 1..];

                let mut r = s[1..].chars();
                if r.next() == Some('.') && r.next().map_or(false, |c| c.is_ascii_digit()) {
                    println!("found period");
                    s = &s[2..];
                    let i = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
                    f = Some(s[0..i].into());
                    s = &s[i - 1..];
                }
                toks.push(Tok::Num(w, f));
            }
            c if c == '_' || c.is_ascii_alphabetic() => {
                let i = s
                    .find(|c: char| c != '_' && !c.is_ascii_alphanumeric())
                    .unwrap_or(s.len());
                let ident = &s[0..i];
                s = &s[i - 1..];
                if let Ok(k) = ident.parse() {
                    toks.push(Tok::Kw(k));
                } else {
                    toks.push(Tok::Ident(ident.into()));
                }
            }
            '"' => return Err(format_err!("I don't know how to parse string literals")),
            c if c.is_ascii_whitespace() => {}
            _ => return Err(format_err!("unrecognized character: {:?}", c)),
        }

        s = &s[c.len_utf8()..];
    }
    Ok(toks)
}

// TODO: Get a better testing framework, even if just Go-style table tests.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kws_parse() {
        assert_eq!(Kw::Progressive, "progressive".parse().unwrap());
        assert_eq!(Kw::Enum, "enum".parse().unwrap());
        assert_eq!(Kw::To, "to".parse().unwrap());
        assert_eq!(Kw::Modify, "modify".parse().unwrap());
    }

    #[test]
    fn bad_kws_fail_parse() {
        assert!("foobar".parse::<Kw>().is_err());
        assert!("Requires".parse::<Kw>().is_err());
        assert!("".parse::<Kw>().is_err());
        assert!("samus".parse::<Kw>().is_err());
    }

    #[test]
    fn kws_display() {
        assert_eq!("alias", format!("{}", Kw::Alias));
        assert_eq!("link", format!("{}", Kw::Link));
        assert_eq!("items", format!("{}", Kw::Items));
        assert_eq!("in", format!("{}", Kw::In));
    }

    #[test]
    fn syms_parse() {
        assert_eq!(Sym::Plus, "+".parse().unwrap());
        assert_eq!(Sym::Dot, ".".parse().unwrap());
        assert_eq!(Sym::RBrace, "}".parse().unwrap());
        assert_eq!(Sym::DoubleArrow, "=>".parse().unwrap());
    }

    #[test]
    fn bad_syms_fail_parse() {
        assert!("\"".parse::<Sym>().is_err());
        assert!("".parse::<Sym>().is_err());
        assert!("++".parse::<Sym>().is_err());
    }

    #[test]
    fn toks_display() {
        assert_eq!("<=", format!("{}", Sym::LE));
        assert_eq!(")", format!("{}", Sym::RParen));
        assert_eq!("*", format!("{}", Sym::Star));
    }

    #[test]
    fn lex_syms() {
        use super::Sym::*;
        use super::Tok::*;

        let str = "=======";
        let toks = vec![Sym(Eq), Sym(Eq), Sym(Eq), Sym(Assign)];
        assert_eq!(toks, lex(str).unwrap());

        let str = "===>>>=!==";
        let toks = vec![
            Sym(Eq),
            Sym(DoubleArrow),
            Sym(GT),
            Sym(GE),
            Sym(NEq),
            Sym(Assign),
        ];
        assert_eq!(toks, lex(str).unwrap());

        let str = "--->+<<==";
        let toks = vec![
            Sym(Minus),
            Sym(Minus),
            Sym(Arrow),
            Sym(Plus),
            Sym(LT),
            Sym(LE),
            Sym(Assign),
        ];
        assert_eq!(toks, lex(str).unwrap());

        let str = "*+-/%.;:,{}()[]";
        let toks = vec![
            Sym(Star),
            Sym(Plus),
            Sym(Minus),
            Sym(Slash),
            Sym(Percent),
            Sym(Dot),
            Sym(Semi),
            Sym(Colon),
            Sym(Comma),
            Sym(LBrace),
            Sym(RBrace),
            Sym(LParen),
            Sym(RParen),
            Sym(LBrack),
            Sym(RBrack),
        ];
        assert_eq!(toks, lex(str).unwrap());

        let str = "- > = > = < = =";
        let toks = vec![
            Sym(Minus),
            Sym(GT),
            Sym(Assign),
            Sym(GT),
            Sym(Assign),
            Sym(LT),
            Sym(Assign),
            Sym(Assign),
        ];
        assert_eq!(toks, lex(str).unwrap());
    }

    #[test]
    fn lex_nums() {
        use super::Sym::*;
        use super::Tok::*;

        let str = "0";
        let toks = vec![Num("0".into(), None)];
        assert_eq!(toks, lex(str).unwrap());

        let str = "1234567890";
        let toks = vec![Num("1234567890".into(), None)];
        assert_eq!(toks, lex(str).unwrap());

        let str = "1f";
        let toks = vec![Num("1".into(), None), Ident("f".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "0.1";
        let toks = vec![Num("0".into(), Some("1".into()))];
        assert_eq!(toks, lex(str).unwrap());

        let str = "99999999999999999999.00000000000000000000";
        let toks = vec![Num(
            "99999999999999999999".into(),
            Some("00000000000000000000".into()),
        )];
        assert_eq!(toks, lex(str).unwrap());

        let str = "1.1.1";
        let toks = vec![
            Num("1".into(), Some("1".into())),
            Sym(Dot),
            Num("1".into(), None),
        ];
        assert_eq!(toks, lex(str).unwrap());

        let str = ".1";
        let toks = vec![Sym(Dot), Num("1".into(), None)];
        assert_eq!(toks, lex(str).unwrap());

        let str = "1 .1";
        let toks = vec![Num("1".into(), None), Sym(Dot), Num("1".into(), None)];
        assert_eq!(toks, lex(str).unwrap());
    }

    #[test]
    fn lex_idents_kws() {
        use super::Kw::*;
        use super::Tok::*;

        let str = "a";
        let toks = vec![Ident("a".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "A";
        let toks = vec![Ident("A".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "z1";
        let toks = vec![Ident("z1".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "_";
        let toks = vec![Ident("_".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "the_quick_brown_fox_jumps_over_the_1234567890_lazy_dogs";
        let toks = vec![Ident(
            "the_quick_brown_fox_jumps_over_the_1234567890_lazy_dogs".into(),
        )];
        assert_eq!(toks, lex(str).unwrap());

        let str = "a b";
        let toks = vec![Ident("a".into()), Ident("b".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "if";
        let toks = vec![Kw(If)];
        assert_eq!(toks, lex(str).unwrap());

        let str = "_if";
        let toks = vec![Ident("_if".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "if9";
        let toks = vec![Ident("if9".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "if then else";
        let toks = vec![Kw(If), Ident("then".into()), Kw(Else)];
        assert_eq!(toks, lex(str).unwrap());
    }

    #[test]
    fn lex_idents_whitespace() {
        use super::Tok::*;

        let str = "  \t\n  \r    ";
        let toks: Vec<Tok> = vec![];
        assert_eq!(toks, lex(str).unwrap());

        let str = "s\tv";
        let toks = vec![Ident("s".into()), Ident("v".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "s\n\r\nq";
        let toks = vec![Ident("s".into()), Ident("q".into())];
        assert_eq!(toks, lex(str).unwrap());
    }

    #[test]
    fn lex_comments() {
        use super::Sym::*;
        use super::Tok::*;

        let str = "foo//bar\nbaz";
        let toks = vec![Ident("foo".into()), Ident("baz".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "foo//bar";
        let toks = vec![Ident("foo".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "//foo\n///bar\n//\n/\n/baz";
        let toks = vec![Sym(Slash), Sym(Slash), Ident("baz".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "foo/*bar*/baz";
        let toks = vec![Ident("foo".into()), Ident("baz".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "foo/*/ */bar\nbaz";
        let toks = vec![
            Ident("foo".into()),
            Ident("bar".into()),
            Ident("baz".into()),
        ];
        assert_eq!(toks, lex(str).unwrap());

        let str = "foo /* /* */ */ bar";
        let toks = vec![Ident("foo".into()), Ident("bar".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "/**/";
        let toks: Vec<Tok> = vec![];
        assert_eq!(toks, lex(str).unwrap());

        let str = "/***/";
        let toks: Vec<Tok> = vec![];
        assert_eq!(toks, lex(str).unwrap());

        let str = "/*********/";
        let toks: Vec<Tok> = vec![];
        assert_eq!(toks, lex(str).unwrap());

        let str = "/*/ bar */";
        let toks: Vec<Tok> = vec![];
        assert_eq!(toks, lex(str).unwrap());

        let str = "/* */ */";
        let toks = vec![Sym(Star), Sym(Slash)];
        assert_eq!(toks, lex(str).unwrap());

        let str = "///*\n*/";
        let toks = vec![Sym(Star), Sym(Slash)];
        assert_eq!(toks, lex(str).unwrap());

        let str = "foo/*/*/*/*/**/*/*/*/*/";
        let toks = vec![Ident("foo".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "/* /* */ /* */ */";
        let toks: Vec<Tok> = vec![];
        assert_eq!(toks, lex(str).unwrap());

        let str = "/* // */\n*/";
        let toks = vec![Sym(Star), Sym(Slash)];
        assert_eq!(toks, lex(str).unwrap());
    }
}
