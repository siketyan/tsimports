use biome_js_factory::make;
use biome_js_syntax::{
    AnyJsBinding, AnyJsCombinedSpecifier, AnyJsImportAssertionEntry, AnyJsImportClause,
    AnyJsModuleSource, AnyJsNamedImportSpecifier, JsDefaultImportSpecifier, JsImport,
    JsImportAssertion, JsImportAssertionEntryList, JsImportBareClause, JsImportCombinedClause,
    JsImportDefaultClause, JsImportNamedClause, JsImportNamespaceClause, JsNamedImportSpecifier,
    JsNamedImportSpecifierList, JsNamedImportSpecifiers, JsNamespaceImportSpecifier,
    JsShorthandNamedImportSpecifier, JsSyntaxKind, JsSyntaxToken,
};
use biome_rowan::{AstSeparatedList, SyntaxResult, TriviaPiece};

fn make_token_with_l_space(kind: JsSyntaxKind) -> JsSyntaxToken {
    if let Some(text) = kind.to_string() {
        JsSyntaxToken::new_detached(kind, &format!(" {text}"), [TriviaPiece::whitespace(1)], [])
    } else {
        panic!("token kind {kind:?} cannot be transformed to text")
    }
}

fn make_token_with_r_space(kind: JsSyntaxKind) -> JsSyntaxToken {
    if let Some(text) = kind.to_string() {
        JsSyntaxToken::new_detached(kind, &format!("{text} "), [], [TriviaPiece::whitespace(1)])
    } else {
        panic!("token kind {kind:?} cannot be transformed to text")
    }
}

pub trait Remake
where
    Self: Sized,
{
    fn remake(&self) -> SyntaxResult<Self>;
}

impl Remake for AnyJsBinding {
    fn remake(&self) -> SyntaxResult<Self> {
        Ok(match self {
            AnyJsBinding::JsBogusBinding(_) => todo!(),
            AnyJsBinding::JsIdentifierBinding(binding) => {
                make::js_identifier_binding(make::ident(binding.name_token()?.text_trimmed()))
                    .into()
            }
            AnyJsBinding::JsMetavariable(binding) => {
                make::js_metavariable(make::ident(binding.value_token()?.text_trimmed())).into()
            }
        })
    }
}

impl Remake for AnyJsModuleSource {
    fn remake(&self) -> SyntaxResult<Self> {
        Ok(match self {
            AnyJsModuleSource::JsMetavariable(source) => {
                make::js_metavariable(make::ident(source.value_token()?.text_trimmed())).into()
            }
            AnyJsModuleSource::JsModuleSource(source) => {
                make::js_module_source(make::ident(source.value_token()?.text_trimmed())).into()
            }
        })
    }
}

impl Remake for AnyJsImportAssertionEntry {
    fn remake(&self) -> SyntaxResult<Self> {
        match self {
            Self::JsBogusImportAssertionEntry(_) => todo!(),
            Self::JsImportAssertionEntry(entry) => Ok(make::js_import_assertion_entry(
                make::ident(entry.key()?.text_trimmed()),
                make_token_with_r_space(JsSyntaxKind::COLON),
                make::ident(entry.value_token()?.text_trimmed()),
            )
            .into()),
        }
    }
}

impl Remake for JsImportAssertionEntryList {
    fn remake(&self) -> SyntaxResult<Self> {
        Ok(make::js_import_assertion_entry_list(
            self.iter()
                .map(|entry| Ok(entry?.remake()?))
                .collect::<Result<Vec<_>, _>>()?,
            self.separators()
                .map(|_| make_token_with_r_space(JsSyntaxKind::COMMA))
                .collect::<Vec<_>>(),
        ))
    }
}

impl Remake for JsImportAssertion {
    fn remake(&self) -> SyntaxResult<Self> {
        Ok(make::js_import_assertion(
            make::token(self.assertion_kind()?.kind()),
            make::token_decorated_with_space(JsSyntaxKind::L_CURLY),
            self.assertions().remake()?,
            make::token_decorated_with_space(JsSyntaxKind::R_CURLY),
        ))
    }
}

impl Remake for JsImportBareClause {
    fn remake(&self) -> SyntaxResult<Self> {
        let mut builder = make::js_import_bare_clause(self.source()?.remake()?);

        if let Some(assertion) = self.assertion() {
            builder = builder.with_assertion(assertion.remake()?);
        }

        Ok(builder.build())
    }
}

impl Remake for AnyJsCombinedSpecifier {
    fn remake(&self) -> SyntaxResult<Self> {
        Ok(match self {
            Self::JsNamedImportSpecifiers(specifiers) => specifiers.remake()?.into(),
            Self::JsNamespaceImportSpecifier(specifier) => specifier.remake()?.into(),
        })
    }
}

impl Remake for JsImportCombinedClause {
    fn remake(&self) -> SyntaxResult<Self> {
        let mut builder = make::js_import_combined_clause(
            self.default_specifier()?.remake()?,
            make_token_with_r_space(JsSyntaxKind::COMMA),
            self.specifier()?.remake()?,
            make::token_decorated_with_space(JsSyntaxKind::FROM_KW),
            self.source()?.remake()?,
        );

        if let Some(assertion) = self.assertion() {
            builder = builder.with_assertion(assertion.remake()?)
        }

        Ok(builder.build())
    }
}

impl Remake for JsDefaultImportSpecifier {
    fn remake(&self) -> SyntaxResult<Self> {
        Ok(make::js_default_import_specifier(
            self.local_name()?.remake()?,
        ))
    }
}

impl Remake for JsImportDefaultClause {
    fn remake(&self) -> SyntaxResult<Self> {
        let mut builder = make::js_import_default_clause(
            self.default_specifier()?.remake()?,
            make::token_decorated_with_space(JsSyntaxKind::FROM_KW),
            self.source()?.remake()?,
        );

        if self.type_token().is_some() {
            builder = builder.with_type_token(make_token_with_r_space(JsSyntaxKind::TYPE_KW))
        }

        if let Some(assertion) = self.assertion() {
            builder = builder.with_assertion(assertion.remake()?);
        }

        Ok(builder.build())
    }
}

impl Remake for JsNamedImportSpecifier {
    fn remake(&self) -> SyntaxResult<Self> {
        let mut builder = make::js_named_import_specifier(
            make::js_literal_export_name(make::ident(self.name()?.value()?.text_trimmed())),
            make::token_decorated_with_space(JsSyntaxKind::AS_KW),
            self.local_name()?.remake()?,
        );

        if self.type_token().is_some() {
            builder = builder.with_type_token(make_token_with_r_space(JsSyntaxKind::TYPE_KW));
        }

        Ok(builder.build())
    }
}

impl Remake for JsShorthandNamedImportSpecifier {
    fn remake(&self) -> SyntaxResult<Self> {
        let mut builder = make::js_shorthand_named_import_specifier(self.local_name()?.remake()?);

        if self.type_token().is_some() {
            builder = builder.with_type_token(make_token_with_r_space(JsSyntaxKind::TYPE_KW));
        }

        Ok(builder.build())
    }
}

impl Remake for AnyJsNamedImportSpecifier {
    fn remake(&self) -> SyntaxResult<Self> {
        match self {
            Self::JsBogusNamedImportSpecifier(_) => todo!(),
            Self::JsNamedImportSpecifier(specifier) => Ok(specifier.remake()?.into()),
            Self::JsShorthandNamedImportSpecifier(specifier) => Ok(specifier.remake()?.into()),
        }
    }
}

impl Remake for JsNamedImportSpecifierList {
    fn remake(&self) -> SyntaxResult<Self> {
        Ok(make::js_named_import_specifier_list(
            {
                let mut items = self
                    .iter()
                    .map(|specifier| specifier?.remake())
                    .collect::<Result<Vec<_>, _>>()?;

                items.sort_by_key(|item| item.imported_name().unwrap().text_trimmed().to_string());
                items
            },
            self.separators()
                .map(|_| make_token_with_r_space(JsSyntaxKind::COMMA))
                .collect::<Vec<_>>(),
        ))
    }
}

impl Remake for JsNamedImportSpecifiers {
    fn remake(&self) -> SyntaxResult<Self> {
        Ok(make::js_named_import_specifiers(
            make_token_with_r_space(JsSyntaxKind::L_CURLY),
            self.specifiers().remake()?,
            make_token_with_l_space(JsSyntaxKind::R_CURLY),
        ))
    }
}

impl Remake for JsImportNamedClause {
    fn remake(&self) -> SyntaxResult<Self> {
        let mut builder = make::js_import_named_clause(
            self.named_specifiers()?.remake()?,
            make::token_decorated_with_space(JsSyntaxKind::FROM_KW),
            self.source()?.remake()?,
        );

        if self.type_token().is_some() {
            builder = builder.with_type_token(make_token_with_r_space(JsSyntaxKind::TYPE_KW));
        }

        Ok(builder.build())
    }
}

impl Remake for JsNamespaceImportSpecifier {
    fn remake(&self) -> SyntaxResult<Self> {
        Ok(make::js_namespace_import_specifier(
            make::token(JsSyntaxKind::STAR),
            make::token_decorated_with_space(JsSyntaxKind::AS_KW),
            self.local_name()?.remake()?,
        ))
    }
}

impl Remake for JsImportNamespaceClause {
    fn remake(&self) -> SyntaxResult<Self> {
        let mut builder = make::js_import_namespace_clause(
            self.namespace_specifier()?.remake()?,
            make::token_decorated_with_space(JsSyntaxKind::FROM_KW),
            self.source()?.remake()?,
        );

        if self.type_token().is_some() {
            builder = builder.with_type_token(make_token_with_r_space(JsSyntaxKind::TYPE_KW));
        }

        if let Some(assertion) = self.assertion() {
            builder = builder.with_assertion(assertion.remake()?);
        }

        Ok(builder.build())
    }
}

impl Remake for JsImport {
    fn remake(&self) -> SyntaxResult<Self> {
        let clause = match self.import_clause()? {
            AnyJsImportClause::JsImportBareClause(clause) => clause.remake()?.into(),
            AnyJsImportClause::JsImportCombinedClause(clause) => clause.remake()?.into(),
            AnyJsImportClause::JsImportDefaultClause(clause) => clause.remake()?.into(),
            AnyJsImportClause::JsImportNamedClause(clause) => clause.remake()?.into(),
            AnyJsImportClause::JsImportNamespaceClause(clause) => clause.remake()?.into(),
        };

        Ok(
            make::js_import(make_token_with_r_space(JsSyntaxKind::IMPORT_KW), clause)
                .with_semicolon_token(make::token(JsSyntaxKind::SEMICOLON))
                .build(),
        )
    }
}
