mod analyze;
mod import_kind;
mod transform;

use biome_js_parser::{parse, JsParserOptions};
use biome_js_syntax::{AnyJsRoot, JsFileSource};
use biome_parser::diagnostic::ParseDiagnostic;
use biome_rowan::{AstNode, BatchMutationExt};
use itertools::Itertools;

use crate::analyze::{collect_imports, group_imports, order_groups};
use crate::transform::Remake;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse the text as a ECMAScript or TypeScript module.")]
    Parser(Vec<ParseDiagnostic>),

    #[error("Only module files (.mjs) or TypeScript files (.ts) are supported.")]
    NotJSModule,
}

pub fn tsimports<'a>(input: impl Into<&'a str>, source: JsFileSource) -> Result<String, Error> {
    let root = parse(input.into(), source, JsParserOptions::default())
        .ok()
        .map_err(Error::Parser)?;

    let AnyJsRoot::JsModule(root) = root else {
        return Err(Error::NotJSModule);
    };

    let mut groups = order_groups(group_imports(collect_imports(&root)));

    groups.iter_mut().for_each(|group| group.reorder_in_place());

    let mut mutation = root.begin();

    for group in &groups {
        for item in group.items() {
            mutation.remove_node(item.to_owned());
        }
    }

    let imports = groups
        .into_iter()
        .map(|group| {
            group
                .items()
                .map(|item| item.remake().unwrap().text())
                .join("\n")
        })
        .join("\n\n");

    Ok([
        imports,
        mutation
            .commit()
            .trim_leading_trivia()
            .unwrap()
            .text()
            .to_string(),
    ]
    .join("\n\n"))
}
