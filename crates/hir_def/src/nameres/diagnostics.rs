//! Diagnostics emitted during DefMap construction.

use cfg::{CfgExpr, CfgOptions};
use hir_expand::MacroCallKind;
use la_arena::Idx;
use syntax::ast;

use crate::{
    item_tree::{self, ItemTreeId},
    nameres::LocalModuleId,
    path::ModPath,
    AstId,
};

#[derive(Debug, PartialEq, Eq)]
pub enum DefDiagnosticKind {
    UnresolvedModule { ast: AstId<ast::Module>, candidate: String },

    UnresolvedExternCrate { ast: AstId<ast::ExternCrate> },

    UnresolvedImport { id: ItemTreeId<item_tree::Import>, index: Idx<ast::UseTree> },

    UnconfiguredCode { ast: AstId<ast::Item>, cfg: CfgExpr, opts: CfgOptions },

    UnresolvedProcMacro { ast: MacroCallKind },

    UnresolvedMacroCall { ast: AstId<ast::MacroCall>, path: ModPath },

    MacroError { ast: MacroCallKind, message: String },

    UnimplementedBuiltinMacro { ast: AstId<ast::Macro> },
}

#[derive(Debug, PartialEq, Eq)]
pub struct DefDiagnostic {
    pub in_module: LocalModuleId,
    pub kind: DefDiagnosticKind,
}

impl DefDiagnostic {
    pub(super) fn unresolved_module(
        container: LocalModuleId,
        declaration: AstId<ast::Module>,
        candidate: String,
    ) -> Self {
        Self {
            in_module: container,
            kind: DefDiagnosticKind::UnresolvedModule { ast: declaration, candidate },
        }
    }

    pub(super) fn unresolved_extern_crate(
        container: LocalModuleId,
        declaration: AstId<ast::ExternCrate>,
    ) -> Self {
        Self {
            in_module: container,
            kind: DefDiagnosticKind::UnresolvedExternCrate { ast: declaration },
        }
    }

    pub(super) fn unresolved_import(
        container: LocalModuleId,
        id: ItemTreeId<item_tree::Import>,
        index: Idx<ast::UseTree>,
    ) -> Self {
        Self { in_module: container, kind: DefDiagnosticKind::UnresolvedImport { id, index } }
    }

    pub(super) fn unconfigured_code(
        container: LocalModuleId,
        ast: AstId<ast::Item>,
        cfg: CfgExpr,
        opts: CfgOptions,
    ) -> Self {
        Self { in_module: container, kind: DefDiagnosticKind::UnconfiguredCode { ast, cfg, opts } }
    }

    pub(super) fn unresolved_proc_macro(container: LocalModuleId, ast: MacroCallKind) -> Self {
        Self { in_module: container, kind: DefDiagnosticKind::UnresolvedProcMacro { ast } }
    }

    pub(super) fn macro_error(
        container: LocalModuleId,
        ast: MacroCallKind,
        message: String,
    ) -> Self {
        Self { in_module: container, kind: DefDiagnosticKind::MacroError { ast, message } }
    }

    pub(super) fn unresolved_macro_call(
        container: LocalModuleId,
        ast: AstId<ast::MacroCall>,
        path: ModPath,
    ) -> Self {
        Self { in_module: container, kind: DefDiagnosticKind::UnresolvedMacroCall { ast, path } }
    }

    pub(super) fn unimplemented_builtin_macro(
        container: LocalModuleId,
        ast: AstId<ast::Macro>,
    ) -> Self {
        Self { in_module: container, kind: DefDiagnosticKind::UnimplementedBuiltinMacro { ast } }
    }
}
