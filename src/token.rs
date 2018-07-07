use failure::{format_err, Error, Fail};
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
                    _ => Err($err{s: s.to_owned()}),
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
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
    Ne <- "!=",
    Lt <- "<",
    Le <- "<=",
    Gt <- ">=",
    Ge <- ">",
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Tok {
    Kw(Kw),
    Sym(Sym),
    Ident(String),
    Num(String, Option<String>),
    String(String),
}

impl fmt::Display for Tok {
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

pub fn lex(mut s: &str) -> Result<Vec<Tok>, Error> {
    let mut toks = Vec::new();
    while s.len() > 0 {
        let c = peek(s).unwrap();
        let rest = &s[1..];

        // Note that we unconditionally advance s at the end of the loop, so
        // multi-character tokens need to advance to their last character,
        // rather than past it.
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
            '/' => toks.push(Tok::Sym(Sym::Slash)),
            '%' => toks.push(Tok::Sym(Sym::Percent)),
            '!' => if peek(rest) == Some('=') {
                s = &s[1..];
                toks.push(Tok::Sym(Sym::Ne));
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
                toks.push(Tok::Sym(Sym::Ge));
            } else {
                toks.push(Tok::Sym(Sym::Gt));
            },
            '<' => if peek(rest) == Some('=') {
                s = &s[1..];
                toks.push(Tok::Sym(Sym::Le));
            } else {
                toks.push(Tok::Sym(Sym::Lt));
            },
            '-' => if peek(rest) == Some('>') {
                s = &s[1..];
                toks.push(Tok::Sym(Sym::Arrow));
            } else {
                toks.push(Tok::Sym(Sym::Minus));
            },
            c if c.is_ascii_digit() => {
                let i = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
                let (w, mut f) = (s[0..i].to_owned(), None);
                s = &s[i - 1..];

                let mut r = s.chars();
                if r.next() == Some('.') && r.next().map_or(false, |c| c.is_ascii_digit()) {
                    s = &s[1..];
                    let i = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
                    f = Some(s[0..i].to_owned());
                    s = &s[i - 1..];
                }
                toks.push(Tok::Num(w, f));
            }
            c @ '_' | c if c.is_ascii_alphabetic() => {
                let i = s.find(|c: char| c != '_' && !c.is_ascii_alphanumeric())
                    .unwrap_or(s.len());
                let ident = &s[0..i];
                s = &s[i - 1..];
                if let Ok(k) = ident.parse() {
                    toks.push(Tok::Kw(k));
                } else {
                    toks.push(Tok::Ident(ident.to_owned()));
                }
            }
            '"' => return Err(format_err!("I don't know how to parse string literals")),
            _ => return Err(format_err!("unrecognized character: {:?}", c)),
        }

        s = &s[c.len_utf8()..];
    }
    Ok(toks)
}

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
        assert_eq!("<=", format!("{}", Sym::Le));
        assert_eq!(")", format!("{}", Sym::RParen));
        assert_eq!("*", format!("{}", Sym::Star));
    }
}
