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
    {$name:ident; err $err:ident; $($kw:ident <- $spl:expr),*,} => {
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
    };
}

toks! { Kw;
    err ParseKwError;
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
    Then <- "then",
    Match <- "match",
    True <- "true",
    False <- "false",
    Not <- "not",
    And <- "and",
    Or <- "or",
    Min <- "min",

    // Prepositions
    With <- "with",
    To <- "to",
    From <- "from",
    In <- "in",
}

toks! { Sym;
    err ParseSymError;
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

/// For a string starting on a block comment marker, advance up to the last
/// character of the block comment. It will recurse in order to handle nested
/// comments.
fn skip_block_comment(mut s: &str) -> Result<&str, Error> {
    assert!(s.len() >= 2);
    assert!(s.starts_with("/*"));
    s = unsafe { s.get_unchecked(2..) };

    // TODO: This feels kind of bad to search twice, but lazy_static/regex is a
    // lot of work for two two-character search patterns. Also this means every
    // recursion is checking for an EOF, which is pointless.
    loop {
        let end = s
            .find("*/")
            .ok_or(format_err!("unterminated block comment"))?;
        match s.find("/*") {
            Some(inner) if inner < end => s = &skip_block_comment(&s[inner..])?,
            _ => break Ok(unsafe { s.get_unchecked(end + 2..) }),
        }
    }
}

/// Lex a string literal, and return the contents (with escapes processed) in the first position,
/// and the remainder of the source in the second. s is expected to already have had the opening quote
/// removed.
fn lex_string_lit<'a>(mut s: &'a str) -> Result<(Cow<'a, str>, &'a str), Error> {
    println!("parsing string literal: {:?}", s);
    // Easy case: there is no escape sequence, so we can just borrow the
    // contents directly.
    let escape = s.find("\\").unwrap_or(s.len());
    let quote = s
        .find("\"")
        .ok_or(format_err!("unterminated string literal"))?;
    if quote < escape {
        return Ok(unsafe {
            (
                s.get_unchecked(0..quote).into(),
                s.get_unchecked(quote + 1..),
            )
        });
    }

    let mut l = String::new();
    while let Some(escape) = s.find("\\") {
        l += unsafe { s.get_unchecked(0..escape) };
        s = unsafe { s.get_unchecked(escape + 1..) };
        match s.chars().next() {
            None => return Err(format_err!("unterminated string literal")),
            Some('"') => l += "\"",
            Some('\\') => l += "\\",
            Some('n') => l += "\n",
            Some('r') => l += "\r",
            Some('t') => l += "\t",
            Some(e) => return Err(format_err!("unrecognized escape sequence: \\{}", e)),
        }
        // Any escape sequence we actually accept is 1 ASCII character long.
        s = unsafe { s.get_unchecked(1..) };
    }
    let quote = s
        .find("\"")
        .ok_or(format_err!("unterminated string literal"))?;
    l += unsafe { s.get_unchecked(0..quote) };
    return Ok((l.into(), unsafe { s.get_unchecked(quote + 1..) }));
}

pub fn lex<'a>(mut s: &'a str) -> Result<Vec<Tok<'a>>, Error> {
    let mut toks = Vec::new();
    while let Some(c) = s.chars().next() {
        let rest = unsafe { s.get_unchecked(c.len_utf8()..) };

        // Note that we unconditionally advance s at the end of the loop, so
        // multi-character tokens need to advance to their last character,
        // rather than past it.
        //
        // Throughout this code, we reborrow s by index past the first character
        // quite frequently. Non-ASCII characters will cause panics.
        //
        // TODO: Mostly the reason for this is having to add -1 in a ton of
        // places so as to avoid having to advance manually in every branch.
        // Should probably think that through more.
        //
        // TODO: Use unsafe to optimize the slicing.
        match c {
            '(' => {
                toks.push(Tok::Sym(Sym::LParen));
                s = rest;
            }
            ')' => {
                toks.push(Tok::Sym(Sym::RParen));
                s = rest;
            }
            '[' => {
                toks.push(Tok::Sym(Sym::LBrack));
                s = rest;
            }
            ']' => {
                toks.push(Tok::Sym(Sym::RBrack));
                s = rest;
            }
            '{' => {
                toks.push(Tok::Sym(Sym::LBrace));
                s = rest;
            }
            '}' => {
                toks.push(Tok::Sym(Sym::RBrace));
                s = rest;
            }
            ';' => {
                toks.push(Tok::Sym(Sym::Semi));
                s = rest;
            }
            ',' => {
                toks.push(Tok::Sym(Sym::Comma));
                s = rest;
            }
            ':' => {
                toks.push(Tok::Sym(Sym::Colon));
                s = rest;
            }
            '.' => {
                toks.push(Tok::Sym(Sym::Dot));
                s = rest;
            }
            '+' => {
                toks.push(Tok::Sym(Sym::Plus));
                s = rest;
            }
            '*' => {
                toks.push(Tok::Sym(Sym::Star));
                s = rest;
            }
            '%' => {
                toks.push(Tok::Sym(Sym::Percent));
                s = rest;
            }
            '/' => match rest.chars().next() {
                Some('/') => {
                    // If we don't find \n, we set i to s.len()-1 so that when we add 1 on the next
                    // line, we end up right at the end of the string.
                    let i = s.find('\n').unwrap_or(s.len() - 1);
                    s = unsafe { s.get_unchecked(i + 1..) };
                }
                Some('*') => s = skip_block_comment(s)?,
                _ => {
                    toks.push(Tok::Sym(Sym::Slash));
                    s = rest;
                }
            },
            '!' => if rest.chars().next() == Some('=') {
                toks.push(Tok::Sym(Sym::NEq));
                s = unsafe { s.get_unchecked(2..) };
            } else {
                return Err(format_err!("expected = after ! to make != token"));
            },
            '=' => match rest.chars().next() {
                Some('=') => {
                    toks.push(Tok::Sym(Sym::Eq));
                    s = unsafe { s.get_unchecked(2..) };
                }
                Some('>') => {
                    toks.push(Tok::Sym(Sym::DoubleArrow));
                    s = unsafe { s.get_unchecked(2..) };
                }
                _ => {
                    toks.push(Tok::Sym(Sym::Assign));
                    s = rest;
                }
            },
            '>' => if rest.chars().next() == Some('=') {
                toks.push(Tok::Sym(Sym::GE));
                s = unsafe { s.get_unchecked(2..) };
            } else {
                toks.push(Tok::Sym(Sym::GT));
                s = rest;
            },
            '<' => if rest.chars().next() == Some('=') {
                toks.push(Tok::Sym(Sym::LE));
                s = unsafe { s.get_unchecked(2..) };
            } else {
                toks.push(Tok::Sym(Sym::LT));
                s = rest;
            },
            '-' => if rest.chars().next() == Some('>') {
                toks.push(Tok::Sym(Sym::Arrow));
                s = unsafe { s.get_unchecked(2..) };
            } else {
                toks.push(Tok::Sym(Sym::Minus));
                s = rest;
            },
            c if c.is_ascii_digit() => {
                let i = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
                let (w, mut f) = (unsafe { s.get_unchecked(0..i).into() }, None);
                s = unsafe { s.get_unchecked(i..) };

                let mut r = s.chars();
                if r.next() == Some('.') && r.next().map_or(false, |c| c.is_ascii_digit()) {
                    s = unsafe { s.get_unchecked(1..) };
                    let i = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
                    f = Some(unsafe { s.get_unchecked(0..i) }.into());
                    s = unsafe { s.get_unchecked(i..) };
                }
                toks.push(Tok::Num(w, f));
            }
            c if c == '_' || c.is_ascii_alphabetic() => {
                let i = s
                    .find(|c: char| c != '_' && !c.is_ascii_alphanumeric())
                    .unwrap_or(s.len());
                let ident = unsafe { s.get_unchecked(0..i) };
                s = unsafe { s.get_unchecked(i..) };
                if let Ok(k) = ident.parse() {
                    toks.push(Tok::Kw(k));
                } else {
                    toks.push(Tok::Ident(ident.into()));
                }
            }
            '"' => {
                let (l, s_) = lex_string_lit(rest)?;
                toks.push(Tok::String(l));
                s = s_;
            }
            c if c.is_ascii_whitespace() => s = rest,
            _ => return Err(format_err!("unrecognized character: {:?}", c)),
        }
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
        use Sym::*;
        use Tok::*;

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
        use Sym::*;
        use Tok::*;

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
        use Kw::*;
        use Tok::*;

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
        use Tok::*;

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
        use Sym::*;
        use Tok::*;

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

    #[test]
    fn lex_string_literals() {
        use Tok::*;

        let str = "\"\"";
        let toks = vec![String("".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "\"abcd\"";
        let toks = vec![String("abcd".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "\"\"\"\"";
        let toks = vec![String("".into()), String("".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "\"\\\"\"";
        let toks = vec![String("\"".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "\"\\\\\"";
        let toks = vec![String("\\".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "\"a\\nb\\rc\\td\"";
        let toks = vec![String("a\nb\rc\td".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "\"a b c \"";
        let toks = vec![String("a b c ".into())];
        assert_eq!(toks, lex(str).unwrap());

        let str = "a\"\"b";
        let toks = vec![Ident("a".into()), String("".into()), Ident("b".into())];
        assert_eq!(toks, lex(str).unwrap());
    }
}
