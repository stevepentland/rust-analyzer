//! When *constructing* `hir`, we start at some parent syntax node and recursively
//! lower the children.
//!
//! This modules allows one to go in the opposite direction: start with a syntax
//! node for a *child*, and get its hir.

use either::Either;

use crate::{
    db::DefDatabase,
    dyn_map::DynMap,
    item_scope::ItemScope,
    keys,
    src::{HasChildSource, HasSource},
    AdtId, AssocItemId, DefWithBodyId, EnumId, EnumVariantId, FieldId, ImplId, Lookup, ModuleDefId,
    ModuleId, TraitId, VariantId,
};

pub trait ChildBySource {
    fn child_by_source(&self, db: &dyn DefDatabase) -> DynMap {
        let mut res = DynMap::default();
        self.child_by_source_to(db, &mut res);
        res
    }
    fn child_by_source_to(&self, db: &dyn DefDatabase, map: &mut DynMap);
}

impl ChildBySource for TraitId {
    fn child_by_source_to(&self, db: &dyn DefDatabase, res: &mut DynMap) {
        let data = db.trait_data(*self);
        for (_name, item) in data.items.iter() {
            match *item {
                AssocItemId::FunctionId(func) => {
                    let src = func.lookup(db).source(db);
                    res[keys::FUNCTION].insert(src, func)
                }
                AssocItemId::ConstId(konst) => {
                    let src = konst.lookup(db).source(db);
                    res[keys::CONST].insert(src, konst)
                }
                AssocItemId::TypeAliasId(ty) => {
                    let src = ty.lookup(db).source(db);
                    res[keys::TYPE_ALIAS].insert(src, ty)
                }
            }
        }
    }
}

impl ChildBySource for ImplId {
    fn child_by_source_to(&self, db: &dyn DefDatabase, res: &mut DynMap) {
        let data = db.impl_data(*self);
        for &item in data.items.iter() {
            match item {
                AssocItemId::FunctionId(func) => {
                    let src = func.lookup(db).source(db);
                    res[keys::FUNCTION].insert(src, func)
                }
                AssocItemId::ConstId(konst) => {
                    let src = konst.lookup(db).source(db);
                    res[keys::CONST].insert(src, konst)
                }
                AssocItemId::TypeAliasId(ty) => {
                    let src = ty.lookup(db).source(db);
                    res[keys::TYPE_ALIAS].insert(src, ty)
                }
            }
        }
    }
}

impl ChildBySource for ModuleId {
    fn child_by_source_to(&self, db: &dyn DefDatabase, res: &mut DynMap) {
        let def_map = self.def_map(db);
        let module_data = &def_map[self.local_id];
        module_data.scope.child_by_source_to(db, res);
    }
}

impl ChildBySource for ItemScope {
    fn child_by_source_to(&self, db: &dyn DefDatabase, res: &mut DynMap) {
        self.declarations().for_each(|item| add_module_def(db, res, item));
        self.unnamed_consts().for_each(|konst| {
            let src = konst.lookup(db).source(db);
            res[keys::CONST].insert(src, konst);
        });
        self.impls().for_each(|imp| add_impl(db, res, imp));
        self.attr_macro_invocs().for_each(|(ast_id, call_id)| {
            let item = ast_id.with_value(ast_id.to_node(db.upcast()));
            res[keys::ATTR_MACRO].insert(item, call_id);
        });

        fn add_module_def(db: &dyn DefDatabase, map: &mut DynMap, item: ModuleDefId) {
            match item {
                ModuleDefId::FunctionId(func) => {
                    let src = func.lookup(db).source(db);
                    map[keys::FUNCTION].insert(src, func)
                }
                ModuleDefId::ConstId(konst) => {
                    let src = konst.lookup(db).source(db);
                    map[keys::CONST].insert(src, konst)
                }
                ModuleDefId::StaticId(statik) => {
                    let src = statik.lookup(db).source(db);
                    map[keys::STATIC].insert(src, statik)
                }
                ModuleDefId::TypeAliasId(ty) => {
                    let src = ty.lookup(db).source(db);
                    map[keys::TYPE_ALIAS].insert(src, ty)
                }
                ModuleDefId::TraitId(trait_) => {
                    let src = trait_.lookup(db).source(db);
                    map[keys::TRAIT].insert(src, trait_)
                }
                ModuleDefId::AdtId(adt) => match adt {
                    AdtId::StructId(strukt) => {
                        let src = strukt.lookup(db).source(db);
                        map[keys::STRUCT].insert(src, strukt)
                    }
                    AdtId::UnionId(union_) => {
                        let src = union_.lookup(db).source(db);
                        map[keys::UNION].insert(src, union_)
                    }
                    AdtId::EnumId(enum_) => {
                        let src = enum_.lookup(db).source(db);
                        map[keys::ENUM].insert(src, enum_)
                    }
                },
                _ => (),
            }
        }
        fn add_impl(db: &dyn DefDatabase, map: &mut DynMap, imp: ImplId) {
            let src = imp.lookup(db).source(db);
            map[keys::IMPL].insert(src, imp)
        }
    }
}

impl ChildBySource for VariantId {
    fn child_by_source_to(&self, db: &dyn DefDatabase, res: &mut DynMap) {
        let arena_map = self.child_source(db);
        let arena_map = arena_map.as_ref();
        for (local_id, source) in arena_map.value.iter() {
            let id = FieldId { parent: *self, local_id };
            match source {
                Either::Left(source) => {
                    res[keys::TUPLE_FIELD].insert(arena_map.with_value(source.clone()), id)
                }
                Either::Right(source) => {
                    res[keys::RECORD_FIELD].insert(arena_map.with_value(source.clone()), id)
                }
            }
        }
    }
}

impl ChildBySource for EnumId {
    fn child_by_source_to(&self, db: &dyn DefDatabase, res: &mut DynMap) {
        let arena_map = self.child_source(db);
        let arena_map = arena_map.as_ref();
        for (local_id, source) in arena_map.value.iter() {
            let id = EnumVariantId { parent: *self, local_id };
            res[keys::VARIANT].insert(arena_map.with_value(source.clone()), id)
        }
    }
}

impl ChildBySource for DefWithBodyId {
    fn child_by_source_to(&self, db: &dyn DefDatabase, res: &mut DynMap) {
        let body = db.body(*self);
        for (_, def_map) in body.blocks(db) {
            // All block expressions are merged into the same map, because they logically all add
            // inner items to the containing `DefWithBodyId`.
            def_map[def_map.root()].scope.child_by_source_to(db, res);
        }
    }
}
