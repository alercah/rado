#![feature(uniform_paths)]

#[macro_use]
extern crate failure;

pub mod ast;
pub mod token;

pub(crate) mod exts;

use ast::{Decl, Stmt};
use failure::{format_err, Error};
use id_map::IdMap;
use mixed_ref::MixedRef;
use std::collections::{HashMap, HashSet};
use string_interner::DefaultStringInterner;

/// An identifier in a Peri program.
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

/// An untyped Id for an entity in a Peri program.
///
/// Ids are only unique within a given type; two entities of different types may
/// have the same Id. Use EntityId instead to get uniqueness.
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Id(id_map::Id);

/// A typed id for an entity in a Peri program. Every entity has a unique
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

/// A trait that abstracts over the various entities in Peri.
pub trait Entity {
    /// Retrieve the entity's parent scope.
    fn parent(&self) -> ScopeId;
}

/// A holder for a dynamic Entity, owned or borrowed.
pub enum DynEntity<'a> {
    Owned(Box<dyn Entity + 'a>),
    Borrowed(&'a (dyn Entity + 'a)),
}

impl<'a> std::ops::Deref for DynEntity<'a> {
    type Target = dyn Entity + 'a;
    fn deref(&self) -> &Self::Target {
        match self {
            DynEntity::Owned(b) => &**b,
            DynEntity::Borrowed(r) => *r,
        }
    }
}

/// An identifier for a scope in a Peri program.
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ScopeId {
    Global,
    Region(Id),
}

/// This is a hack to allow private methods on some traits. If it is used on a
/// trait method, then the trait is sealed and the method cannot be called
/// outside this module, because you cannot construct the hack.
pub struct PrivateHack(());

/// A trait that abstracts over the various scopes in Peri. Almost all
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

/// A Peri program represents the abstract state of a Peri program prior to it
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
        self.global_decls.get(&i).map(|&e| e)
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
    pub fn lookup_ident(&self, scope: &dyn Scope, ident: Ident) -> Option<EntityId> {
        scope
            .lookup_ident(ident)
            .or_else(|| self.lookup_ident(self.get_scope(scope.parent()?).unwrap(), ident))
    }

    /// Lookup an entity by full path. Lookup is done by looking up the first
    /// identifier in the scope with lookup_ident, then each successive
    /// identifier in the path is looked up in the scope found previously.
    pub fn lookup_entity(&self, scope: &dyn Scope, path: &Path) -> Result<EntityId, Error> {
        let mut segs = path.0.iter();
        let mut cur = self
            .lookup_ident(scope, *segs.next().unwrap())
            .ok_or(format_err!("first identifier in path not found in lookup"))?;
        for next in segs {
            let child: &dyn Scope;
            match cur {
                EntityId::Region(r) => child = self.regions.get(r.0).unwrap(),
                _ => return Err(format_err!("tried to lookup entity in non-scope")),
            }
            cur = child
                .lookup_ident(*next)
                .ok_or(format_err!("next segment not found in child scope"))?;
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

/// A Peri region.
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
        self.children.get(&i).map(|&e| e)
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
        // First pass: load all the entities, so that name lookup becomes
        // possible. Properties are not loaded.
        for s in f.stmts {
            match s {
                Stmt::Decl(Decl::Region(r)) => self.add_region(ScopeId::Global, r)?,
                Stmt::Decl(Decl::Item(i)) => self.add_item(ScopeId::Global, i, HashSet::new())?,
                Stmt::Decl(Decl::Items(i)) => self.add_items(ScopeId::Global, i)?,
                _ => {}
            }
        }
        Ok(self.0)
    }

    fn add_region(&mut self, parent: ScopeId, region: ast::Region) -> Result<(), Error> {
        let n = self.add_name(region.name);
        self.validate_name_collisions(parent, n.ident)?;

        let r = Region {
            parent: parent,
            name: n,
            children: HashMap::new(),
        };
        let n = r.name.ident;
        let id = self.regions.insert(r);
        self.get_scope_mut(parent).unwrap().insert_child(
            n,
            EntityId::Region(Id(id)),
            PrivateHack(()),
        );

        let id = ScopeId::Region(Id(id));
        // In the first pass, we only process statements.
        for s in region.stmts {
            match s {
                Stmt::Decl(Decl::Region(r)) => self.add_region(id, r)?,
                Stmt::Decl(Decl::Item(i)) => self.add_item(id, i, HashSet::new())?,
                Stmt::Decl(Decl::Items(i)) => self.add_items(id, i)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn add_item(
        &mut self,
        parent: ScopeId,
        item: ast::Item,
        tags: HashSet<Ident>,
    ) -> Result<(), Error> {
        let n = self.add_name(item.name);
        self.validate_name_collisions(parent, n.ident)?;

        let i = Item {
            parent: parent,
            name: n,
            tags: tags,
        };

        // We put properties off until the second pass ordinarily, except that
        // tags actually declare the tag names, so process them now.
        for s in item.stmts {
            Stmt::Prop(Prop::Tag(t))
        }

        Ok(())
    }
    fn add_items(&mut self, parent: ScopeId, i: ast::Items) -> Result<(), Error> {
        Ok(())
    }

    fn add_ident(&mut self, i: ast::Ident) -> Ident {
        Ident(self.idents.get_or_intern(i.0))
    }
    fn add_name(&mut self, n: ast::DeclName) -> Name {
        Name {
            ident: self.add_ident(n.ident),
            human: n.human,
        }
    }

    fn add_tag(&mut self, t: Ident) -> Result<(), Error> {
        let mut to_check = vec![&**self as &dyn Scope];
        while let Some(s) = to_check.pop() {
            for (n, e) in s.children() {
                if *n == t {
                    return Err(format_err!("tag declared with same name as entity"));
                }
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
        if let Some(e) = self.lookup_ident(self.get_scope(s).unwrap(), n) {
            if self.get_entity(e).unwrap().parent() == s {
                Err(format_err!("name already declared in same scope"))
            } else {
                Err(format_err!("name shadows entity in higher scope"))
            }
        } else {
            Ok(())
        }
    }
}
