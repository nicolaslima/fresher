//! Measurement shim: same public API as before, but routed through `inty`
//! and `inty-bundle` instead of oxc. The goal is to compile a release
//! binary with no `oxc_*` linkage so we can compare binary sizes.
//! Functional fidelity is NOT preserved — many call sites would surface
//! different output at runtime. This is *only* for the size measurement.

use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Plugin sources are post-migration assumed to be JS already; transpile
/// is a pass-through. Inty's parser is invoked once for validation so
/// the inty code path links into the binary.
pub fn transpile_typescript(source: &str, _filename: &str) -> Result<String> {
    // Reference inty's parser so it links. Errors are non-fatal here.
    let _ = inty::parser::parse(source);
    Ok(source.to_string())
}

/// Replaces oxc's `IsolatedDeclarations`. Routes through inty's parser
/// so the inty code path is exercised; declaration emission for a real
/// migration would call `inty::declarations::emit_declarations` against
/// a `CheckedModule`. For binary-size purposes, returning an empty
/// `.d.ts` is fine — what matters is the symbol graph linked in.
pub fn emit_isolated_declarations(source: &str, _filename: &str) -> Result<String> {
    match inty::parser::parse(source) {
        Ok(_) => Ok(String::new()),
        Err(e) => Err(anyhow!("inty parse failed: {:?}", e)),
    }
}

/// Lightweight string heuristic — no parser involvement. Same as before.
pub fn has_es_module_syntax(source: &str) -> bool {
    let has_imports = source.contains("import ") && source.contains(" from ");
    let has_exports = source.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("export ")
    });
    has_imports || has_exports
}

pub fn has_es_imports(source: &str) -> bool {
    source.contains("import ") && source.contains(" from ")
}

pub fn extract_plugin_dependencies(source: &str) -> Vec<String> {
    let prefix = "fresh:plugin/";
    let mut deps = Vec::new();
    let mut seen = HashSet::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("import ") || !trimmed.contains(prefix) {
            continue;
        }
        if let Some(from_idx) = trimmed.find(" from ") {
            let after_from = &trimmed[from_idx + 6..];
            let after_from = after_from.trim();
            let quote_char = after_from.chars().next();
            if let Some(q) = quote_char {
                if q == '"' || q == '\'' {
                    if let Some(end) = after_from[1..].find(q) {
                        let module_path = &after_from[1..1 + end];
                        if let Some(plugin_name) = module_path.strip_prefix(prefix) {
                            if !plugin_name.is_empty() && seen.insert(plugin_name.to_string()) {
                                deps.push(plugin_name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    deps
}

pub fn topological_sort_plugins(
    plugin_names: &[String],
    dependencies: &std::collections::HashMap<String, Vec<String>>,
) -> Result<Vec<String>> {
    use std::collections::HashMap;

    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

    for name in plugin_names {
        in_degree.entry(name.as_str()).or_insert(0);
    }

    for name in plugin_names {
        if let Some(deps) = dependencies.get(name) {
            for dep in deps {
                if in_degree.contains_key(dep.as_str()) {
                    *in_degree.entry(name.as_str()).or_insert(0) += 1;
                    dependents
                        .entry(dep.as_str())
                        .or_default()
                        .push(name.as_str());
                } else {
                    return Err(anyhow!(
                        "Plugin '{}' depends on '{}', which is not installed or not enabled",
                        name,
                        dep
                    ));
                }
            }
        }
    }

    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&name, _)| name)
        .collect();
    queue.sort();

    let mut result: Vec<String> = Vec::with_capacity(plugin_names.len());

    while let Some(current) = queue.first().copied() {
        queue.remove(0);
        result.push(current.to_string());

        if let Some(deps) = dependents.get(current) {
            let mut newly_ready = Vec::new();
            for &dependent in deps {
                if let Some(deg) = in_degree.get_mut(dependent) {
                    *deg -= 1;
                    if *deg == 0 {
                        newly_ready.push(dependent);
                    }
                }
            }
            newly_ready.sort();
            queue.extend(newly_ready);
            queue.sort();
        }
    }

    if result.len() != plugin_names.len() {
        let in_result: HashSet<&str> = result.iter().map(|s| s.as_str()).collect();
        let cycle_plugins: Vec<String> = plugin_names
            .iter()
            .filter(|n| !in_result.contains(n.as_str()))
            .cloned()
            .collect();
        return Err(anyhow!(
            "Plugin dependency cycle detected among: {}",
            cycle_plugins.join(", ")
        ));
    }

    Ok(result)
}

/// Bundle via `inty-bundle::bundle`. Drops the source map (callers
/// today consume `String`, not `BundleOutput`).
pub fn bundle_module(entry_path: &Path) -> Result<String> {
    match inty_bundle::bundle(entry_path) {
        Ok(out) => Ok(out.code),
        Err(e) => Err(anyhow!("inty-bundle failed: {}", e)),
    }
}

/// Crude string-level strip. Real migration would route through inty's
/// parser + a re-emitter; for measurement purposes the symbol graph
/// established by `transpile_typescript`'s `inty::parser::parse` call
/// is enough to link inty in.
pub fn strip_imports_and_exports(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("import ") {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("export ") {
            out.push_str(rest);
            out.push('\n');
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

// Keep these module-private types so external dependents that use them
// (none, currently — they're crate-private) keep compiling.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ModuleMetadata {
    path: PathBuf,
    var_name: String,
}
