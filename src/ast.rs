use lalrpop_util::lalrpop_mod;
use serde_derive::{Deserialize, Serialize};

lalrpop_mod!(#[allow(clippy::all)] pub parse, "/ast/parse.rs");

use std::vec::Vec;

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct File {
  pub stmts: Vec<Stmt>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Ident(pub String);

pub type Num = num_rational::BigRational;
pub type Path = Vec<Ident>;

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct DeclName {
  pub ident: Ident,
  pub human: Option<String>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub enum ModVec<T> {
  New(Vec<T>),
  Mod(Vec<(bool, T)>),
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub enum Stmt {
  Decl(Decl),
  Prop(Prop),
  Cond(Expr, Vec<Stmt>, Vec<Stmt>),
  Modify(Decl),
  Override(Decl),
  Delete(Decl),
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Region {
  pub name: DeclName,
  pub stmts: Vec<Stmt>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub enum LinkDir {
  To,
  From,
  With,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Link {
  pub name: Option<DeclName>,
  pub dir: LinkDir,
  pub regions: ModVec<Path>,
  pub stmts: Vec<Stmt>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Item {
  pub name: DeclName,
  pub stmts: Vec<Stmt>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Items {
  pub tags: ModVec<Ident>,
  pub items: Vec<Item>,
  pub nested: Vec<Items>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Location {
  pub name: DeclName,
  pub stmts: Vec<Stmt>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Locations {
  pub decls: Vec<Location>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Param {
  pub name: Ident,
  pub ty: Option<Ty>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct FnDecl {
  pub name: DeclName,
  pub params: Vec<Param>,
  pub ret_ty: Option<Ty>,
  pub body: Expr,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Enum {
  pub name: DeclName,
  pub variants: Vec<DeclName>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Config {
  pub name: DeclName,
  pub ty: Ty,
  pub default: Option<Expr>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct ConfigEnum {
  pub name: DeclName,
  pub variants: Vec<DeclName>,
  pub default: Option<Expr>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct TypedConfig {
  pub name: DeclName,
  pub default: Option<Expr>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Configs {
  pub ty: Ty,
  pub configs: Vec<TypedConfig>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Configset {
  pub name: DeclName,
  pub vals: Vec<(Path, Expr)>,
  pub configsets: Vec<Path>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Random {
  pub name: DeclName,
  pub vals: ModVec<Expr>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub enum Decl {
  Region(Region),
  Link(Link),
  Item(Item),
  Items(Items),
  Location(Location),
  Locations(Locations),
  Fn(FnDecl),
  Enum(Enum),
  Config(Config),
  ConfigEnum(ConfigEnum),
  Configs(Configs),
  Configset(Configset),
  Random(Random),
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Requires {
  pub cond: Expr,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Visible {
  pub cond: Expr,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Unlock {
  pub item: Path,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Tag {
  pub tags: ModVec<Ident>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Alias {
  pub names: ModVec<Ident>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Provides {
  pub items: ModVec<Path>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Progressive {
  pub items: ModVec<Path>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Val {
  pub name: Ident,
  pub ty: Option<Ty>,
  pub val: Expr,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Max {
  pub expr: Expr,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Avail {
  pub items: ModVec<(bool, Path, Option<Num>)>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Grants {
  pub items: ModVec<(bool, Path)>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct StartWith {
  pub items: Vec<Path>,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct StartIn {
  pub region: Path,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub enum Prop {
  Requires(Requires),
  Visible(Visible),
  Unlock(Unlock),
  Tag(Tag),
  Alias(Alias),
  Provides(Provides),
  Progressive(Progressive),
  Val(Val),
  Max(Max),
  Consumable,
  Avail(Avail),
  Grants(Grants),
  StartWith(StartWith),
  StartIn(StartIn),
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub enum BinOp {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Eq,
  NEq,
  LT,
  LE,
  GT,
  GE,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub enum Builtin {
  Count,
  Max,
  Min,
  Sum,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct MatchArm {
  pub pat: Path,
  pub expr: Expr,
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub enum Expr {
  Grouped(Box<Expr>),
  Num(Num),
  Bool(bool),
  List(Vec<Expr>),
  Name(Path),
  Call(Box<Expr>, Vec<Expr>),
  Builtin(Builtin, Vec<Expr>),
  Not(Box<Expr>),
  Bin(Box<Expr>, BinOp, Box<Expr>),
  And(Vec<Expr>),
  Or(Vec<Expr>),
  If(Box<Expr>, Box<Expr>, Box<Expr>),
  Match(Box<Expr>, Vec<MatchArm>),
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub enum Ty {
  Num,
  Bool,
  Item,
  Fn(Vec<Ty>, Box<Ty>),
  List(Box<Ty>),
  Name(Path),
}

#[cfg(test)]
mod test {
  use super::*;
  use failure::{format_err, Error};
  use std::{fs, path};

  macro_rules! sample {
    ($name:ident, $file:expr) => {
      #[test]
      fn $name() -> Result<(), Error> {
        let sample_path = path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/", $file));
        let gold_path = sample_path.with_extension("gold.rson");
        let parse_path = sample_path.with_extension("parse.rson");

        let sample = fs::read_to_string(&sample_path).map_err(|e| {
          format_err!(
            "Error reading sample file {}: {}",
            sample_path.to_string_lossy(),
            e
          )
        })?;
        let tokens = crate::token::lex(&sample).unwrap();
        let parsed = parse::FileParser::new().parse(tokens).unwrap();

        let parsed_rson = rson_rs::ser::pretty::to_string(&parsed)
          .map_err(|e| format_err!("Error serializing parsed AST to RSON: {}", e))?;
        match fs::write(&parse_path, &parsed_rson) {
          Ok(()) => println!(
            "Successfully wrote parsed AST as RSON to {}",
            parse_path.to_string_lossy()
          ),
          Err(e) => println!(
            "Error writing parsed AST to {}: {}. Continuing anyway.",
            parse_path.to_string_lossy(),
            e
          ),
        }

        let gold = fs::read_to_string(&gold_path).map_err(|e| {
          format_err!(
            "Error reading gold RSON file {}: {}",
            gold_path.to_string_lossy(),
            e
          )
        })?;

        let diff = difference::Changeset::new(&gold, &parsed_rson, "\n");
        if diff.distance > 0 {
          println!("{}", diff);
          panic!("Parsed AST does not match gold file! See diff printed above for differences.");
        }
        Ok(())
      }
    };
  }

  sample!(sample_alttp_items, "samples/alttp/items.rado");
  sample!(sample_alttp_regions, "samples/alttp/regions.rado");
  sample!(sample_alttp_config, "samples/alttp/config.rado");
}
