use biome_js_syntax::{AnyJsModuleItem, JsImport, JsModule};
use itertools::Itertools;

use crate::import_kind::ImportKind;

pub fn collect_imports(module: &JsModule) -> Vec<JsImport> {
    module
        .items()
        .into_iter()
        .filter_map(|item| match item {
            AnyJsModuleItem::JsImport(import) => Some(import),
            _ => None,
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct ImportGroup {
    kind: ImportKind,
    items: Vec<JsImport>,
}

impl ImportGroup {
    fn new(kind: ImportKind, items: Vec<JsImport>) -> Self {
        Self { kind, items }
    }

    pub fn items(&self) -> impl Iterator<Item = &JsImport> {
        self.items.iter()
    }

    pub fn reorder_in_place(&mut self) {
        self.items
            .sort_by_key(|import| import.source_text().unwrap().to_string())
    }
}

pub fn group_imports(imports: impl IntoIterator<Item = JsImport>) -> Vec<ImportGroup> {
    imports
        .into_iter()
        .into_group_map_by(|import| ImportKind::guess(import.source_text().unwrap().text()))
        .into_iter()
        .map(|(kind, imports)| ImportGroup::new(kind, imports))
        .collect()
}

pub fn order_groups(groups: impl IntoIterator<Item = ImportGroup>) -> Vec<ImportGroup> {
    groups
        .into_iter()
        .sorted_by_key(|group| group.kind)
        .collect()
}
