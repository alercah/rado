use std::vec::Vec;

pub struct File {
    pub stmts: Vec<Stmt>,
}

pub struct Ident(String);

pub enum Name {
    Global,
    Sub(Box<Name>, Ident),
    Id(Ident),
}

pub struct DeclName {
    pub ident: Ident,
    pub human: String,
}

pub enum Stmt {
    Decl(DeclName, Decl),
    Prop(Prop),
    Cond(Expr, Vec<Stmt>),
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
    pub name: DeclName,
    pub dir: LinkDir,
    pub regions: Vec<Name>,
    pub stmts: Vec<Stmt>,
}

pub struct Item {
    pub name: DeclName,
    pub stmts: Vec<Stmt>,
}

pub struct Items {
    pub tags: Vec<Ident>,
    pub decls: Vec<Item>,
}

pub struct Location {
    pub name: DeclName,
    pub stmts: Vec<Stmt>,
}

pub struct Locations {
    pub decls: Vec<Location>,
}

pub struct Enum {
    pub name: DeclName,
    pub ids: Vec<DeclName>,
}

pub struct Param {
    pub name: Ident,
    pub ty: Option<Ty>,
}
pub struct Fn {
    pub name: DeclName,
    pub params: Vec<Param>,
    pub ret_ty: Option<Ty>,
    pub body: Expr,
}

pub struct Config {
    pub name: DeclName,
    pub ty: Ty,
    pub default: Option<Expr>,
}

pub struct ConfigEnum {
    pub name: DeclName,
    pub e_decl: Enum,
    pub default: Option<Expr>,
}

pub struct Configset {
    pub name: DeclName,
    pub vals: Vec<(Name, Expr)>,
}

pub struct Random {
    pub name: DeclName,
    pub vals: Vec<Expr>,
}

pub enum Decl {
    Region(Region),
    Link(Link),
    Item(Item),
    Items(Items),
    Location(Location),
    Locations(Locations),
    Fn(Fn),
    Enum(Enum),
    Config(Config),
    ConfigEnum(ConfigEnum),
    Random(Random),
}

pub enum Prop {
    Requirement {},
    Visibility {},
    Unlock {},
    Tag {},
    Alias {},
    Provides {},
    Progressive {},
    Value {},
    Max {},
    Consumable {},
    Restrict {},
    Availability {},
    Grant {},
    Quantity {},
    StartWith {},
    StartIn {},
}

pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

pub enum Builtin {
    Count,
    Max,
    Min,
}

pub struct MatchArm {
    pub pat: Name,
    pub expr: Expr,
}

pub enum Expr {
    NumLit(),
    BoolLit(bool),
    List(Vec<Expr>),
    Name(Name),
    FnCall(Name, Vec<Expr>),
    BuiltinCall(Builtin, Vec<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Match(Box<Expr>, Vec<MatchArm>),
}

pub enum Ty {
    Num,
    Bool,
    Item,
    Fn(Vec<Ty>, Box<Ty>),
    List(Box<Ty>),
    Name(Ident),
}
