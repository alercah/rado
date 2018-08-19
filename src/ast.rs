mod parse;

use std::vec::Vec;

pub struct File {
    pub stmts: Vec<Stmt>,
}

pub struct Ident(String);

pub type Num = num_rational::BigRational;

pub enum Name {
    Global,
    Sub(Box<Name>, Ident),
    Id(Ident),
}

pub struct DeclName {
    pub ident: Ident,
    pub human: Option<String>,
}

pub enum ModVec<T> {
    New(Vec<T>),
    Mod(Vec<(bool, T)>),
}

pub enum Stmt {
    Decl(Decl),
    Prop(Prop),
    Cond(Expr, Vec<Stmt>, Vec<Stmt>),
    Modify(Decl),
    Override(Decl),
    Delete(Decl),
}

pub struct Region {
    pub name: DeclName,
    pub stmts: Vec<Stmt>,
}

pub enum LinkDir {
    To,
    From,
    With,
}
pub struct Link {
    pub name: Option<DeclName>,
    pub dir: LinkDir,
    pub regions: ModVec<Name>,
    pub stmts: Vec<Stmt>,
}

pub struct Item {
    pub name: DeclName,
    pub stmts: Vec<Stmt>,
}

pub struct Items {
    pub tags: ModVec<Ident>,
    pub decls: Vec<Item>,
}

pub struct Location {
    pub name: DeclName,
    pub stmts: Vec<Stmt>,
}

pub struct Locations {
    pub decls: Vec<Location>,
}

pub struct Param {
    pub name: Ident,
    pub ty: Option<Ty>,
}
pub struct FnDecl {
    pub name: DeclName,
    pub params: Vec<Param>,
    pub ret_ty: Option<Ty>,
    pub body: Expr,
}

pub struct Enum {
    pub name: DeclName,
    pub variants: Vec<DeclName>,
}

pub struct Config {
    pub name: DeclName,
    pub ty: Ty,
    pub default: Option<Expr>,
}

pub struct ConfigEnum {
    pub name: DeclName,
    pub variants: Vec<DeclName>,
    pub default: Option<Expr>,
}

pub struct Configset {
    pub name: DeclName,
    pub vals: Vec<(Name, Expr)>,
}

pub struct Random {
    pub name: DeclName,
    pub vals: ModVec<Expr>,
}

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
    Random(Random),
}

pub struct Requires {
    pub cond: Expr,
}

pub struct Visible {
    pub cond: Expr,
}

pub struct Unlock {
    pub item: Name,
}

pub struct Tag {
    pub tags: ModVec<Ident>,
}

pub struct Alias {
    pub names: ModVec<Ident>,
}

pub struct Provides {
    pub items: ModVec<Name>,
}

pub struct Progressive {
    pub items: ModVec<Name>,
}

pub struct Val {
    pub name: Ident,
    pub ty: Option<Ty>,
    pub val: Expr,
}

pub struct Max {
    pub expr: Expr,
}

pub struct Restrict {
    pub entities: ModVec<(bool, Name)>,
}

pub struct Avail {
    pub items: ModVec<(bool, Name, Option<Num>)>,
}

pub struct Grants {
    pub items: ModVec<(bool, Name)>,
}

pub struct Count {
    pub count: Num,
}

pub struct StartWith {
    pub items: Vec<Name>,
}

pub struct StartIn {
    pub region: Name,
}

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
    Restrict(Restrict),
    Avail(Avail),
    Grants(Grants),
    Count(Count),
    StartWith(StartWith),
    StartIn(StartIn),
}

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

pub enum Builtin {
    Count,
    Max,
    Min,
    Sum,
}

pub struct MatchArm {
    pub pat: Name,
    pub expr: Expr,
}

pub enum Expr {
    Grouped(Box<Expr>),
    Num(Num),
    Bool(bool),
    List(Vec<Expr>),
    Name(Name),
    Call(Box<Expr>, Vec<Expr>),
    Builtin(Builtin, Vec<Expr>),
    Not(Box<Expr>),
    Bin(Box<Expr>, BinOp, Box<Expr>),
    And(Vec<Expr>),
    Or(Vec<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Match(Box<Expr>, Vec<MatchArm>),
}

pub enum Ty {
    Num,
    Bool,
    Item,
    Fn(Vec<Ty>, Box<Ty>),
    List(Box<Ty>),
    Name(Name),
}
