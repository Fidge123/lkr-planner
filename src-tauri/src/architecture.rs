//! An area may only use areas of lower layers, which rules out cycles between areas.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use syn::visit::{self, Visit};

const LAYERS: &[&[&str]] = &[
    &["error", "http", "secret_manager"],
    &["telemetry"],
    &["local_store"],
    &["daylite", "holidays", "zep"],
    &["calendar"],
];

#[test]
fn areas_use_only_lower_layers() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let files = rust_files(&src);
    let test_modules: Vec<Vec<String>> = files
        .iter()
        .flat_map(|file| test_module_declarations(&module_path(&src, file), file))
        .collect();
    let is_test_code = |path: &[String]| test_modules.iter().any(|m| path.starts_with(m));

    let mut violations = BTreeSet::new();
    for file in &files {
        let module = module_path(&src, file);
        let Some(area) = area_of(&module) else {
            continue;
        };
        if is_test_code(&module) {
            continue;
        }
        let layer = layer_of(area).unwrap_or_else(|| panic!("{area} is in no layer"));
        for used in used_paths(file, &module) {
            let Some(used_area) = area_of(&used) else {
                continue;
            };
            if used_area == area || is_test_code(&used) {
                continue;
            }
            let used_layer =
                layer_of(used_area).unwrap_or_else(|| panic!("{used_area} is in no layer"));
            if used_layer >= layer {
                violations.insert(format!("{} uses {}", module.join("::"), used.join("::")));
            }
        }
    }
    assert!(violations.is_empty(), "{violations:#?}");
}

fn layer_of(area: &str) -> Option<usize> {
    LAYERS.iter().position(|layer| layer.contains(&area))
}

/// The crate root wires every area together and belongs to none.
fn area_of(module: &[String]) -> Option<&str> {
    match module {
        [first, area, ..] if first == "integrations" => Some(area),
        [first, ..] if first != "integrations" => Some(first),
        _ => None,
    }
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .unwrap()
        .flat_map(|entry| {
            let path = entry.unwrap().path();
            if path.is_dir() {
                rust_files(&path)
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                vec![path]
            } else {
                vec![]
            }
        })
        .collect()
}

fn module_path(src: &Path, file: &Path) -> Vec<String> {
    let mut module: Vec<String> = file
        .strip_prefix(src)
        .unwrap()
        .with_extension("")
        .iter()
        .map(|segment| segment.to_string_lossy().into_owned())
        .collect();
    if matches!(
        module.last().map(String::as_str),
        Some("mod" | "lib" | "main")
    ) {
        module.pop();
    }
    module
}

fn parse(file: &Path) -> syn::File {
    syn::parse_file(&std::fs::read_to_string(file).unwrap())
        .unwrap_or_else(|error| panic!("{}: {error}", file.display()))
}

fn is_test_only(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg")
            && attr
                .parse_args::<syn::Path>()
                .is_ok_and(|path| path.is_ident("test"))
    })
}

fn test_module_declarations(module: &[String], file: &Path) -> Vec<Vec<String>> {
    parse(file)
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Mod(declared)
                if declared.content.is_none() && is_test_only(&declared.attrs) =>
            {
                Some([module, &[declared.ident.to_string()]].concat())
            }
            _ => None,
        })
        .collect()
}

/// Resolves `crate::` and `super::` paths outside of test code to absolute module paths.
fn used_paths(file: &Path, module: &[String]) -> Vec<Vec<String>> {
    let mut collector = PathCollector {
        module,
        paths: vec![],
    };
    collector.visit_file(&parse(file));
    collector.paths
}

struct PathCollector<'a> {
    module: &'a [String],
    paths: Vec<Vec<String>>,
}

impl PathCollector<'_> {
    fn record(&mut self, segments: Vec<String>) {
        let ups = segments.iter().take_while(|s| *s == "super").count();
        let resolved = match segments.first().map(String::as_str) {
            Some("crate") => segments[1..].to_vec(),
            Some("super") if ups <= self.module.len() => {
                [&self.module[..self.module.len() - ups], &segments[ups..]].concat()
            }
            _ => return,
        };
        self.paths.push(resolved);
    }

    fn record_use(&mut self, prefix: &mut Vec<String>, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(path) => {
                prefix.push(path.ident.to_string());
                self.record_use(prefix, &path.tree);
                prefix.pop();
            }
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.record_use(prefix, tree);
                }
            }
            syn::UseTree::Name(name) => {
                self.record([prefix.as_slice(), &[name.ident.to_string()]].concat())
            }
            syn::UseTree::Rename(rename) => {
                self.record([prefix.as_slice(), &[rename.ident.to_string()]].concat())
            }
            syn::UseTree::Glob(_) => self.record(prefix.clone()),
        }
    }
}

impl<'ast> Visit<'ast> for PathCollector<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let attrs = match item {
            syn::Item::Fn(item) => &item.attrs,
            syn::Item::Impl(item) => &item.attrs,
            syn::Item::Mod(item) => &item.attrs,
            syn::Item::Use(item) => &item.attrs,
            syn::Item::Struct(item) => &item.attrs,
            syn::Item::Enum(item) => &item.attrs,
            syn::Item::Const(item) => &item.attrs,
            syn::Item::Static(item) => &item.attrs,
            _ => return visit::visit_item(self, item),
        };
        if !is_test_only(attrs) {
            visit::visit_item(self, item);
        }
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.record_use(&mut vec![], &item.tree);
    }

    fn visit_field(&mut self, field: &'ast syn::Field) {
        if !is_test_only(&field.attrs) {
            visit::visit_field(self, field);
        }
    }

    fn visit_stmt(&mut self, stmt: &'ast syn::Stmt) {
        let attrs = match stmt {
            syn::Stmt::Local(local) => &local.attrs,
            syn::Stmt::Macro(mac) => &mac.attrs,
            _ => return visit::visit_stmt(self, stmt),
        };
        if !is_test_only(attrs) {
            visit::visit_stmt(self, stmt);
        }
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.record(path.segments.iter().map(|s| s.ident.to_string()).collect());
        visit::visit_path(self, path);
    }
}
