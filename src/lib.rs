#![warn(clippy::all)]
#![feature(copied)]

pub mod ast;
pub mod token;

pub(crate) mod exts;

use ast::{Decl, ModVec, Prop, Stmt};
use failure::{format_err, Error};
use id_map::IdMap;
use mixed_ref::MixedRef;
use std::collections::{HashMap, HashSet};
use string_interner::DefaultStringInterner;

/// An identifier in a Rado program.
///
/// Identifiers are interned within a single Program, so they cannot be directly
/// converted to strings.
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Ident(string_interner::Sym);

/// A name of a declared entity.
///
/// In addition to an identifier, an entity's name may have a human-readable
/// alternate, used when formatting messages.
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Name {
  pub ident: Ident,
  pub human: Option<String>,
}

/// A path is a series of names to be used to lookup an entity or value. Paths
/// are always nonempty.
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Path(Vec<Ident>);

impl Path {
  /// Construct a new path. Returns an error if `segments` is empty.
  pub fn new(segments: Vec<Ident>) -> Result<Path, Error> {
    if segments.is_empty() {
      Err(format_err!("trying to construct empty path"))
    } else {
      Ok(Path(segments))
    }
  }
}

/// An untyped Id for an entity in a Rado program.
///
/// Ids are only unique within a given type; two entities of different types may
/// have the same Id. Use EntityId instead to get uniqueness.
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Id(id_map::Id);

impl From<Id> for id_map::Id {
  fn from(i: Id) -> id_map::Id {
    i.0
  }
}

/// A typed id for an entity in a Rado program. Every entity has a unique
/// EntityId.
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum EntityId {
  Region(Id),
  Item(Id),
  /// Because tags have no additional information other than their identifier,
  /// they use an identifier directly as the id. The corresponding Tag struct
  /// type is used only for implementing the Entity trait.
  Tag(Ident),
}

macro_rules! unwrap_entity_id {
  ($name:ident, $variant:ident, $ty:ident) => {
    /// Unwrap this `EntityId` to the specific variant.
    ///
    /// # Panics
    ///
    /// Panics if `self` is a different variant.
    pub fn $name(&self) -> $ty {
      match self {
        EntityId::$variant(x) => *x,
        _ => panic!("Entity id is not a {}: {:?}", stringify!($variant), self),
      }
    }
  };
}

impl EntityId {
  unwrap_entity_id!(unwrap_region, Region, Id);
  unwrap_entity_id!(unwrap_item, Item, Id);
  unwrap_entity_id!(unwrap_tag, Tag, Ident);
}

/// A trait that abstracts over the various entities in Rado.
pub trait Entity {
  /// Retrieve the entity's parent scope.
  fn parent(&self) -> ScopeId;
}

/// An identifier for a scope in a Rado program.
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ScopeId {
  Global,
  Region(Id),
}

/// This is a hack to allow private methods on some traits. If it is used on a
/// trait method, then the trait is sealed and the method cannot be called
/// outside this module, because you cannot construct the hack.
pub struct PrivateHack(());

/// A trait that abstracts over the various scopes in Rado. Almost all
/// implementations are also entities, but Program implements the global scope.
pub trait Scope {
  /// Retrieve the scope's parent. Only the global scope does not have a parent.
  fn parent(&self) -> Option<ScopeId>;
  /// Lookup a single identifier in this scope. To do lookup across a scope
  /// tree, which is more usual, use methods of Program.
  fn lookup_ident(&self, i: Ident) -> Option<EntityId>;
  /// Get an iterator over all children of this scope.
  fn children<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a Ident, &'a EntityId)> + 'a>;

  /// Private method.
  fn insert_child(&mut self, i: Ident, e: EntityId, _: PrivateHack);
}

/// A Rado program represents the abstract state of a Rado program prior to it
/// being configured or used.
pub struct Program {
  items: IdMap<Item>,
  regions: IdMap<Region>,
  global_decls: HashMap<Ident, EntityId>,
  idents: DefaultStringInterner,
}

impl Scope for Program {
  fn parent(&self) -> Option<ScopeId> {
    None
  }
  fn lookup_ident(&self, i: Ident) -> Option<EntityId> {
    self.global_decls.get(&i).copied()
  }
  fn insert_child(&mut self, i: Ident, e: EntityId, _: PrivateHack) {
    if self.global_decls.insert(i, e).is_some() {
      panic!("overwrote existing entity when inserting new one");
    }
  }
  fn children<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a Ident, &'a EntityId)> + 'a> {
    Box::new(self.global_decls.iter())
  }
}

impl Default for Program {
  /// Construct a completely empty program.
  fn default() -> Program {
    Program {
      items: IdMap::new(),
      regions: IdMap::new(),
      global_decls: HashMap::new(),
      idents: DefaultStringInterner::new(),
    }
  }
}

impl Program {
  /// Construct a program from an AST.
  pub fn from_ast(file: ast::File) -> Result<Program, Error> {
    FromAST(Program::default()).build(file)
  }

  /// Lookup a single identifier in a scope. Lookup proceeds by traversing
  /// upwards along the scope tree to find if any scopes contain the provided
  /// identifier.
  pub fn lookup(&self, scope: &dyn Scope, ident: Ident) -> Option<EntityId> {
    scope
      .lookup_ident(ident)
      .or_else(|| self.lookup(self.get_scope(scope.parent()?).unwrap(), ident))
  }

  /// Lookup an entity by full path. Lookup is done by looking up the first
  /// identifier in the scope with lookup_ident, then each successive
  /// identifier in the path is looked up in the scope found previously.
  pub fn lookup_entity(&self, scope: &dyn Scope, path: &Path) -> Result<EntityId, Error> {
    let mut segs = path.0.iter();
    let mut cur = self
      .lookup(scope, *segs.next().unwrap())
      .ok_or_else(|| format_err!("first identifier in path not found in lookup"))?;
    for next in segs {
      let child: &dyn Scope;
      match cur {
        EntityId::Region(r) => child = self.regions.get(r.0).unwrap(),
        _ => return Err(format_err!("tried to lookup entity in non-scope")),
      }
      cur = child
        .lookup_ident(*next)
        .ok_or_else(|| format_err!("next segment not found in child scope"))?;
    }
    Ok(cur)
  }

  /// Find the entity with the provided id.
  pub fn get_entity(&self, e: EntityId) -> Option<MixedRef<dyn Entity>> {
    match e {
      EntityId::Region(r) => self
        .regions
        .get(r.0)
        .map(|e| MixedRef::Borrowed(e as &dyn Entity)),
      EntityId::Item(i) => self
        .items
        .get(i.0)
        .map(|e| MixedRef::Borrowed(e as &dyn Entity)),
      EntityId::Tag(t) => Some(MixedRef::Owned(Box::new(Tag(t)))),
    }
  }

  /// Find the scope with the provided id.
  pub fn get_scope(&self, s: ScopeId) -> Option<&dyn Scope> {
    match s {
      ScopeId::Global => Some(self),
      ScopeId::Region(r) => self.regions.get(r.0).map(|e| e as &dyn Scope),
    }
  }

  /// Find the scope with the provided id, and return a mutable reference.
  pub fn get_scope_mut(&mut self, s: ScopeId) -> Option<&mut dyn Scope> {
    match s {
      ScopeId::Global => Some(self),
      ScopeId::Region(r) => self.regions.get_mut(r.0).map(|e| e as &mut dyn Scope),
    }
  }
}

/// A Rado region.
pub struct Region {
  parent: ScopeId,
  name: Name,
  children: HashMap<Ident, EntityId>,
}

impl Entity for Region {
  fn parent(&self) -> ScopeId {
    self.parent
  }
}

impl Scope for Region {
  fn parent(&self) -> Option<ScopeId> {
    Some(self.parent)
  }
  fn lookup_ident(&self, i: Ident) -> Option<EntityId> {
    self.children.get(&i).copied()
  }
  fn insert_child(&mut self, i: Ident, e: EntityId, _: PrivateHack) {
    if self.children.insert(i, e).is_some() {
      panic!("overwrote existing entity when inserting new one");
    }
  }
  fn children<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a Ident, &'a EntityId)> + 'a> {
    Box::new(self.children.iter())
  }
}

/// A item to be randomized.
pub struct Item {
  parent: ScopeId,
  name: Name,
  tags: HashSet<Ident>,
}

impl Entity for Item {
  fn parent(&self) -> ScopeId {
    self.parent
  }
}

/// A tag is just an identifier. This struct exists only to give an
/// implementation of Entity for tags.
pub struct Tag(pub Ident);

impl Entity for Tag {
  fn parent(&self) -> ScopeId {
    ScopeId::Global
  }
}

// Wrapper struct to organize all the code that loads an AST into one place and
// avoid polluting the Program method namespace.
struct FromAST(Program);

impl std::ops::Deref for FromAST {
  type Target = Program;
  fn deref(&self) -> &Program {
    &self.0
  }
}

impl std::ops::DerefMut for FromAST {
  fn deref_mut(&mut self) -> &mut Program {
    &mut self.0
  }
}

impl FromAST {
  fn build(mut self, f: ast::File) -> Result<Program, Error> {
    self.populate_scope(ScopeId::Global, &f.stmts)?;
    self.build_scope(ScopeId::Global, f.stmts)?;
    Ok(self.0)
  }

  fn populate_scope(&mut self, scope: ScopeId, stmts: &[ast::Stmt]) -> Result<(), Error> {
    // First pass: load all the entities, so that name lookup becomes
    // possible. Tags are the only properties loaded.
    for s in stmts {
      match s {
        Stmt::Decl(Decl::Region(r)) => self.add_region(scope, &r)?,
        Stmt::Decl(Decl::Item(i)) => self.add_item(scope, &i)?,
        Stmt::Decl(Decl::Items(i)) => self.add_items(scope, &i)?,
        _ => unimplemented!(),
      }
    }
    Ok(())
  }

  fn add_region(&mut self, parent: ScopeId, region: &ast::Region) -> Result<(), Error> {
    let n = self.add_name(&region.name);
    self.validate_name_collisions(parent, n.ident)?;

    let r = Region {
      parent,
      name: n,
      children: HashMap::new(),
    };
    let n = r.name.ident;
    let id = self.regions.insert(r);
    self
      .get_scope_mut(parent)
      .unwrap()
      .insert_child(n, EntityId::Region(Id(id)), PrivateHack(()));

    self.populate_scope(ScopeId::Region(Id(id)), &region.stmts)?;
    Ok(())
  }

  fn add_item(&mut self, parent: ScopeId, item: &ast::Item) -> Result<(), Error> {
    let n = self.add_name(&item.name);
    self.validate_name_collisions(parent, n.ident)?;

    // We put properties off until the second pass ordinarily, except that
    // tags actually declare the tag names, so we process them now.
    let i = Item {
      parent,
      name: n,
      tags: HashSet::new(),
    };
    for s in &item.stmts {
      if let Stmt::Prop(Prop::Tag(t)) = s {
        self.add_tag_vec(&t.tags)?;
      }
    }
    let n = i.name.ident;
    let id = self.items.insert(i);
    self
      .get_scope_mut(parent)
      .unwrap()
      .insert_child(n, EntityId::Item(Id(id)), PrivateHack(()));
    Ok(())
  }
  fn add_items(&mut self, parent: ScopeId, items: &ast::Items) -> Result<(), Error> {
    self.add_tag_vec(&items.tags)?;
    for i in &items.items {
      self.add_item(parent, &i)?;
    }
    for i in &items.nested {
      self.add_items(parent, &i)?;
    }
    Ok(())
  }

  fn add_ident(&mut self, i: &ast::Ident) -> Ident {
    Ident(self.idents.get_or_intern(&*i.0))
  }
  // add_name adds an identifier into the interning cache and returns a Name
  // from it. It does not set the human name; that must be done manually
  // during the second pass.
  fn add_name(&mut self, n: &ast::DeclName) -> Name {
    Name {
      ident: self.add_ident(&n.ident),
      human: None,
    }
  }

  fn add_tag_vec(&mut self, tags: &ModVec<ast::Ident>) -> Result<(), Error> {
    for t in match tags {
      ModVec::New(v) => either::Left(v.iter()),
      ModVec::Mod(v) => either::Right(v.iter().map(|p| &p.1)),
    } {
      self.add_tag(&t)?;
    }
    Ok(())
  }
  fn add_tag(&mut self, tag: &ast::Ident) -> Result<(), Error> {
    let t = self.add_ident(tag);
    let mut to_check = vec![&**self as &dyn Scope];
    while let Some(s) = to_check.pop() {
      for (n, e) in s.children() {
        if *n == t {
          if e == &EntityId::Tag(t) {
            return Ok(());
          }
          return Err(format_err!("tag declared with same name as entity"));
        }
        #[allow(clippy::single_match)]
        match e {
          EntityId::Region(r) => to_check.push(self.regions.get(r.0).unwrap()),
          _ => {}
        }
      }
    }
    self.global_decls.insert(t, EntityId::Tag(t));
    Ok(())
  }

  fn validate_name_collisions(&self, s: ScopeId, n: Ident) -> Result<(), Error> {
    if let Some(e) = self.lookup(self.get_scope(s).unwrap(), n) {
      if self.get_entity(e).unwrap().parent() == s {
        Err(format_err!("name already declared in same scope"))
      } else {
        Err(format_err!("name shadows entity in higher scope"))
      }
    } else {
      Ok(())
    }
  }

  fn build_scope(&mut self, scope: ScopeId, stmts: Vec<ast::Stmt>) -> Result<(), Error> {
    for s in stmts {
      match s {
        Stmt::Decl(Decl::Region(region)) => {
          let id = self
            .lookup_ast(self.get_scope(scope).unwrap(), &region.name.ident)
            .unwrap_region();
          self.build_region(id, region)?;
        }
        Stmt::Decl(Decl::Item(item)) => {
          let id = self
            .lookup_ast(self.get_scope(scope).unwrap(), &item.name.ident)
            .unwrap_item();
          self.build_item(id, item, HashSet::new())?;
        }
        Stmt::Decl(Decl::Items(items)) => self.build_items(items, scope, HashSet::new())?,
        _ => unimplemented!(),
      }
    }
    Ok(())
  }

  fn build_region(&mut self, region: Id, input: ast::Region) -> Result<(), Error> {
    self.build_scope(ScopeId::Region(region), input.stmts)?;
    let region = self.regions.get_mut(region.into()).unwrap();
    region.name.human = input.name.human;
    Ok(())
  }

  fn build_item(&mut self, item: Id, input: ast::Item, tags: HashSet<Ident>) -> Result<(), Error> {
    let item = self.items.get_mut(item.into()).unwrap();
    item.name.human = input.name.human;
    item.tags = tags;
    unimplemented!()
  }

  fn build_items(
    &mut self,
    items: ast::Items,
    scope: ScopeId,
    mut tags: HashSet<Ident>,
  ) -> Result<(), Error> {
    guard::guard!(let ModVec::New(t) = items.tags
                      else { unimplemented!() });
    tags.extend(t.into_iter().map(|tag| self.convert_ident(&tag)));

    for nested in items.nested {
      self.build_items(nested, scope, tags.clone())?;
    }
    for item in items.items {
      let id = self
        .lookup_ast(self.get_scope(scope).unwrap(), &item.name.ident)
        .unwrap_item();
      self.build_item(id, item, tags.clone())?;
    }
    Ok(())
  }

  fn lookup_ast(&self, scope: &dyn Scope, ident: &ast::Ident) -> EntityId {
    scope.lookup_ident(self.convert_ident(ident)).unwrap()
  }

  fn convert_ident(&self, ident: &ast::Ident) -> Ident {
    Ident(self.idents.get(&ident.0).unwrap())
  }
}
