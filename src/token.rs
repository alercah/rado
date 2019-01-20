use crate::exts::GetDebug;
use failure::{format_err, Error, Fail};
use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

/// An error from failing to parse a [Kw].
#[derive(Clone, Debug, Fail)]
#[fail(display = "{:?} is not a keyword", s)]
pub struct ParseKwError {
  s: String,
}

/// An error from failing to parse a [Sym].
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

/// Rado keywords.
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
  Configs <- "configs",
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
  Sum <- "sum",

  // Miscellaneous
  With <- "with",
  To <- "to",
  From <- "from",
  In <- "in",
  Default <- "default",
}

/// Rado symbol tokens. Each operator is a distinct token, so some tokens are
/// multiple characters long.
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

/// The sign of a numeric literal. Zero is considered positive, since the minus
/// sign is not used for zero literals; negative 0 is an error.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Sign {
  Positive,
  Negative,
}

impl fmt::Display for Sign {
  /// Display the sign as it renders before a number: nothing if it is
  /// positive, and a minus sign for negative.
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(match self {
      Sign::Positive => "",
      Sign::Negative => "-",
    })
  }
}

/// A Rado token.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Tok<'a> {
  /// A keyword.
  Kw(Kw),
  /// A symbol or operator.
  Sym(Sym),
  /// An identifier other than a keyword.
  Ident(Cow<'a, str>),
  /// A numeric literal. Numbers are unparsed
  Num(
    /// The sign of the number.
    Sign,
    /// The whole-number portion of the number (before the '.', if any).
    Cow<'a, str>,
    /// The decimal portion of the number (after the '.').
    Option<Cow<'a, str>>,
  ),
  /// A string literal. The field contains the string with escapes already
  /// procesed.
  String(Cow<'a, str>),
}

impl<'a> Tok<'a> {
  /// Convert any `Cow` strings owned by this token to owned versions, making
  /// copies if needed. After this, calling `to_owned` on them will be a
  /// no-op.
  pub fn into_owned(self) -> Tok<'static> {
    use Tok::*;
    match self {
      Kw(k) => Kw(k),
      Sym(s) => Sym(s),
      Ident(i) => Ident(Cow::Owned(i.into_owned())),
      Num(s, w, d) => Num(
        s,
        Cow::Owned(w.into_owned()),
        d.map(|d| Cow::Owned(d.into_owned())),
      ),
      String(s) => String(Cow::Owned(s.into_owned())),
    }
  }
}

impl<'a> fmt::Display for Tok<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Tok::Kw(k) => write!(f, "{}", k),
      Tok::Sym(s) => write!(f, "{}", s),
      Tok::Ident(i) => write!(f, "{}", i),
      Tok::Num(s, w, d) => write!(
        f,
        "{}{}{}{}",
        s,
        w,
        if d.is_some() { "." } else { "" },
        d.as_ref().map_or("", |d| &d),
      ),
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
  s = unsafe { s.get_debug_checked(2..) };

  // TODO: This feels kind of bad to search twice, but lazy_static/regex is a
  // lot of work for two two-character search patterns. Also this means every
  // recursion is checking for an EOF, which is pointless.
  loop {
    let end = s
      .find("*/")
      .ok_or_else(|| format_err!("unterminated block comment"))?;
    match s.find("/*") {
      Some(inner) if inner < end => s = &skip_block_comment(&s[inner..])?,
      _ => break Ok(unsafe { s.get_debug_checked(end + 2..) }),
    }
  }
}

/// Lex a numeric literal.
#[allow(clippy::type_complexity, clippy::many_single_char_names)]
fn lex_num_lit(mut s: &str) -> Result<(Cow<'_, str>, Option<Cow<'_, str>>, &str), Error> {
  let i = s
    .find(|c: char| !c.is_ascii_digit())
    .unwrap_or_else(|| s.len());
  let (w, mut f) = (unsafe { s.get_debug_checked(0..i).into() }, None);
  s = unsafe { s.get_debug_checked(i..) };

  let mut r = s.chars();
  if r.next() == Some('.') && r.next().map_or(false, |c| c.is_ascii_digit()) {
    s = unsafe { s.get_debug_checked(1..) };
    let i = s
      .find(|c: char| !c.is_ascii_digit())
      .unwrap_or_else(|| s.len());
    f = Some(unsafe { s.get_debug_checked(0..i) }.into());
    s = unsafe { s.get_debug_checked(i..) };
  }
  if s.chars().next().map_or(false, |c| c.is_ascii_alphabetic()) {
    Err(format_err!("alphabetic character in numeric literal"))?;
  }
  Ok((w, f, s))
}

/// Lex a string literal, and return the contents (with escapes processed) in the first position,
/// and the remainder of the source in the second. s is expected to already have had the opening quote
/// removed.
fn lex_string_lit(mut s: &str) -> Result<(Cow<'_, str>, &str), Error> {
  // Easy case: there is no escape sequence, so we can just borrow the
  // contents directly.
  let escape = s.find('\\').unwrap_or_else(|| s.len());
  let quote = s
    .find('\"')
    .ok_or_else(|| format_err!("unterminated string literal"))?;
  if quote < escape {
    return Ok(unsafe {
      (
        s.get_debug_checked(0..quote).into(),
        s.get_debug_checked(quote + 1..),
      )
    });
  }

  let mut l = String::new();
  while let Some(escape) = s.find('\\') {
    l += unsafe { s.get_debug_checked(0..escape) };
    s = unsafe { s.get_debug_checked(escape + 1..) };
    match s.chars().next() {
      None => Err(format_err!("unterminated string literal"))?,
      Some('"') => l += "\"",
      Some('\\') => l += "\\",
      Some('n') => l += "\n",
      Some('r') => l += "\r",
      Some('t') => l += "\t",
      Some(e) => Err(format_err!("unrecognized escape sequence: \\{}", e))?,
    }
    // Any escape sequence we actually accept is 1 ASCII character long.
    s = unsafe { s.get_debug_checked(1..) };
  }
  let quote = s
    .find('\"')
    .ok_or_else(|| format_err!("unterminated string literal"))?;
  l += unsafe { s.get_debug_checked(0..quote) };
  Ok((l.into(), unsafe { s.get_debug_checked(quote + 1..) }))
}

/// Lex a string into a token vector. An error occurs if the string is not made of legal tokens.
pub fn lex<'a>(mut s: &'a str) -> Result<Vec<Tok<'a>>, Error> {
  let mut toks = Vec::new();
  while let Some(c) = s.chars().next() {
    let rest = unsafe { s.get_debug_checked(c.len_utf8()..) };
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
          s = unsafe { s.get_debug_checked(i + 1..) };
        }
        Some('*') => s = skip_block_comment(s)?,
        _ => {
          toks.push(Tok::Sym(Sym::Slash));
          s = rest;
        }
      },
      '!' => {
        if rest.starts_with('=') {
          toks.push(Tok::Sym(Sym::NEq));
          s = unsafe { s.get_debug_checked(2..) };
        } else {
          Err(format_err!("expected = after ! to make != token"))?;
        }
      }
      '=' => match rest.chars().next() {
        Some('=') => {
          toks.push(Tok::Sym(Sym::Eq));
          s = unsafe { s.get_debug_checked(2..) };
        }
        Some('>') => {
          toks.push(Tok::Sym(Sym::DoubleArrow));
          s = unsafe { s.get_debug_checked(2..) };
        }
        _ => {
          toks.push(Tok::Sym(Sym::Assign));
          s = rest;
        }
      },
      '>' => {
        if rest.starts_with('=') {
          toks.push(Tok::Sym(Sym::GE));
          s = unsafe { s.get_debug_checked(2..) };
        } else {
          toks.push(Tok::Sym(Sym::GT));
          s = rest;
        }
      }
      '<' => {
        if rest.starts_with('=') {
          toks.push(Tok::Sym(Sym::LE));
          s = unsafe { s.get_debug_checked(2..) };
        } else {
          toks.push(Tok::Sym(Sym::LT));
          s = rest;
        }
      }
      '-' => match rest.chars().next() {
        Some('>') => {
          toks.push(Tok::Sym(Sym::Arrow));
          s = unsafe { s.get_debug_checked(2..) };
        }
        Some(c) if c.is_ascii_digit() => {
          let (w, f, s_) = lex_num_lit(rest)?;
          if w.chars().all(|c| c == '0')
            && f.as_ref().unwrap_or(&"".into()).chars().all(|c| c == '0')
          {
            return Err(format_err!("negative zero numeric literal"));
          }
          toks.push(Tok::Num(Sign::Negative, w, f));
          s = s_;
        }
        _ => {
          toks.push(Tok::Sym(Sym::Minus));
          s = rest;
        }
      },
      c if c.is_ascii_digit() => {
        let (w, f, s_) = lex_num_lit(s)?;
        toks.push(Tok::Num(Sign::Positive, w, f));
        s = s_;
      }
      c if c == '_' || c.is_ascii_alphabetic() => {
        let i = s
          .find(|c: char| c != '_' && !c.is_ascii_alphanumeric())
          .unwrap_or_else(|| s.len());
        let ident = unsafe { s.get_debug_checked(0..i) };
        s = unsafe { s.get_debug_checked(i..) };
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
      _ => Err(format_err!("unrecognized character: {:?}", c))?,
    }
  }
  Ok(toks)
}

// TODO: Get a better testing framework, even if just Go-style table tests.
#[cfg(test)]
mod tests {
  use super::*;
  use proptest::{proptest, proptest_helper};

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
    use self::Sym::*;
    use self::Tok::*;

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
    use self::Sym::*;
    use self::Tok::*;

    let str = "0";
    let toks = vec![Num(Sign::Positive, "0".into(), None)];
    assert_eq!(toks, lex(str).unwrap());

    let str = "1234567890";
    let toks = vec![Num(Sign::Positive, "1234567890".into(), None)];
    assert_eq!(toks, lex(str).unwrap());

    let str = "0.1";
    let toks = vec![Num(Sign::Positive, "0".into(), Some("1".into()))];
    assert_eq!(toks, lex(str).unwrap());

    let str = "99999999999999999999.00000000000000000000";
    let toks = vec![Num(
      Sign::Positive,
      "99999999999999999999".into(),
      Some("00000000000000000000".into()),
    )];
    assert_eq!(toks, lex(str).unwrap());

    let str = "1.1.1";
    let toks = vec![
      Num(Sign::Positive, "1".into(), Some("1".into())),
      Sym(Dot),
      Num(Sign::Positive, "1".into(), None),
    ];
    assert_eq!(toks, lex(str).unwrap());

    let str = ".1";
    let toks = vec![Sym(Dot), Num(Sign::Positive, "1".into(), None)];
    assert_eq!(toks, lex(str).unwrap());

    let str = "1 .1";
    let toks = vec![
      Num(Sign::Positive, "1".into(), None),
      Sym(Dot),
      Num(Sign::Positive, "1".into(), None),
    ];
    assert_eq!(toks, lex(str).unwrap());

    let str = "-1";
    let toks = vec![Num(Sign::Negative, "1".into(), None)];
    assert_eq!(toks, lex(str).unwrap());

    let str = "-2.2";
    let toks = vec![Num(Sign::Negative, "2".into(), Some("2".into()))];
    assert_eq!(toks, lex(str).unwrap());

    let str = "-0.1";
    let toks = vec![Num(Sign::Negative, "0".into(), Some("1".into()))];
    assert_eq!(toks, lex(str).unwrap());

    let str = "0.-1";
    let toks = vec![
      Num(Sign::Positive, "0".into(), None),
      Sym(Dot),
      Num(Sign::Negative, "1".into(), None),
    ];
    assert_eq!(toks, lex(str).unwrap());
  }

  #[test]
  fn lex_idents_kws() {
    use self::Kw::*;
    use self::Tok::*;

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

    let str = "if than else";
    let toks = vec![Kw(If), Ident("than".into()), Kw(Else)];
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
    use self::Sym::*;
    use self::Tok::*;

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

  #[test]
  fn lex_errors() {
    let str = "!";
    assert!(lex(str).is_err());

    let str = "\0";
    assert!(lex(str).is_err());

    let str = "\x12";
    assert!(lex(str).is_err());

    let str = "é";
    assert!(lex(str).is_err());

    let str = "=!";
    assert!(lex(str).is_err());

    let str = "\u{1034a}";
    assert!(lex(str).is_err());

    let str = "\u{ffef}hi";
    assert!(lex(str).is_err());
  }

  proptest! {
      #[test]
      fn always_valid(ref s in ".*") {
          let _ = lex(s);
      }
  }
}
