use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateSpec {
    pub relative_path: PathBuf,
    pub template: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkloadContract {
    pub package_count: u64,
    pub regular_file_count: u64,
    pub max_package_depth: u32,
    pub min_directory_count: u64,
    pub dominant_file_types: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkloadRecipe {
    pub name: String,
    pub description: String,
    pub root_package_name: String,
    pub root_packages: usize,
    pub fanout_per_depth: Vec<usize>,
    pub scoped_package_period: usize,
    pub scopes: Vec<String>,
    pub root_templates: Vec<TemplateSpec>,
    pub package_templates: Vec<TemplateSpec>,
    pub expected_contract: WorkloadContract,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct WorkloadSummary {
    pub name: String,
    pub recipe_path: String,
    pub package_count: u64,
    pub regular_file_count: u64,
    pub directory_count: u64,
    pub total_bytes: u64,
    pub max_package_depth: u32,
    pub dominant_file_types: Vec<String>,
}

#[derive(Debug, Clone)]
struct LoadedTemplate {
    relative_path: PathBuf,
    contents: String,
}

#[derive(Debug, Clone)]
struct PackagePlan {
    index: usize,
    depth: u32,
    name: String,
    safe_name: String,
    relative_dir: PathBuf,
    dependency_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BenchmarkWorkload {
    recipe_path: PathBuf,
    recipe: WorkloadRecipe,
    root_templates: Vec<LoadedTemplate>,
    package_templates: Vec<LoadedTemplate>,
}

pub fn default_workload_recipe_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benches/workloads/node-modules-100k/recipe.json")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn display_recipe_path(recipe_path: &Path) -> String {
    recipe_path
        .strip_prefix(repo_root())
        .map(|relative| relative.display().to_string())
        .unwrap_or_else(|_| recipe_path.display().to_string())
}

impl BenchmarkWorkload {
    pub fn load_default() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let workload = Self::from_recipe_path(&default_workload_recipe_path())?;
        workload.ensure_default_benchmark_contract()?;
        Ok(workload)
    }

    pub fn from_recipe_path(
        recipe_path: &Path,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let recipe_dir = recipe_path
            .parent()
            .ok_or_else(|| format!("benchmark workload recipe has no parent: {}", recipe_path.display()))?
            .to_path_buf();
        let recipe: WorkloadRecipe = serde_json::from_slice(&std::fs::read(recipe_path)?)?;
        let root_templates = load_templates(&recipe.root_templates, &recipe_dir)?;
        let package_templates = load_templates(&recipe.package_templates, &recipe_dir)?;
        let workload = Self {
            recipe_path: recipe_path.to_path_buf(),
            recipe,
            root_templates,
            package_templates,
        };
        workload.validate_contract()?;
        Ok(workload)
    }

    pub fn name(&self) -> &str {
        &self.recipe.name
    }

    pub fn recipe_path(&self) -> &Path {
        &self.recipe_path
    }

    pub fn description(&self) -> &str {
        &self.recipe.description
    }

    pub fn expected_contract(&self) -> &WorkloadContract {
        &self.recipe.expected_contract
    }

    pub fn planned_summary(&self) -> Result<WorkloadSummary, Box<dyn std::error::Error + Send + Sync>> {
        let root_packages = self.root_package_names();
        let packages = self.build_package_plan();
        let mut directory_tracker = DirectoryTracker::default();
        let mut regular_file_count = 0u64;
        let mut total_bytes = 0u64;

        for template in &self.root_templates {
            let rendered = render_template(
                &template.contents,
                &[
                    ("ROOT_PACKAGE_NAME", self.recipe.root_package_name.clone()),
                    (
                        "ROOT_DEPENDENCIES_JSON",
                        json_dependency_entries(&root_packages, 4),
                    ),
                ],
            );
            regular_file_count += 1;
            total_bytes += rendered.len() as u64;
            directory_tracker.track_parent_of(&template.relative_path);
        }

        for package in &packages {
            let package_context = package_context(package);
            for template in &self.package_templates {
                let rendered = render_template(&template.contents, &package_context);
                regular_file_count += 1;
                total_bytes += rendered.len() as u64;
                directory_tracker.track_parent_of(&package.relative_dir.join(&template.relative_path));
            }
        }

        Ok(WorkloadSummary {
            name: self.recipe.name.clone(),
            recipe_path: display_recipe_path(&self.recipe_path),
            package_count: packages.len() as u64,
            regular_file_count,
            directory_count: directory_tracker.len(),
            total_bytes,
            max_package_depth: packages.iter().map(|package| package.depth).max().unwrap_or(0),
            dominant_file_types: self.recipe.expected_contract.dominant_file_types.clone(),
        })
    }

    pub fn materialize(
        &self,
        input_root: &Path,
    ) -> Result<WorkloadSummary, Box<dyn std::error::Error + Send + Sync>> {
        if input_root.exists() {
            std::fs::remove_dir_all(input_root)?;
        }
        std::fs::create_dir_all(input_root)?;

        let root_packages = self.root_package_names();
        for template in &self.root_templates {
            let rendered = render_template(
                &template.contents,
                &[
                    ("ROOT_PACKAGE_NAME", self.recipe.root_package_name.clone()),
                    (
                        "ROOT_DEPENDENCIES_JSON",
                        json_dependency_entries(&root_packages, 4),
                    ),
                ],
            );
            let output_path = input_root.join(&template.relative_path);
            write_file(&output_path, &rendered)?;
        }

        for package in self.build_package_plan() {
            let context = package_context(&package);
            for template in &self.package_templates {
                let rendered = render_template(&template.contents, &context);
                let output_path = input_root.join(&package.relative_dir).join(&template.relative_path);
                write_file(&output_path, &rendered)?;
            }
        }

        let summary = summarize_generated_tree(input_root, self)?;
        self.ensure_summary_matches_contract(&summary)?;
        Ok(summary)
    }

    pub fn validate_contract(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let contract = &self.recipe.expected_contract;
        if contract.package_count == 0 {
            return Err("benchmark workload contract must declare at least one package".into());
        }
        if contract.max_package_depth < 2 {
            return Err("benchmark workload contract must declare nested package depth".into());
        }
        if contract.min_directory_count == 0 {
            return Err("benchmark workload contract must declare a positive directory-count floor".into());
        }
        if self.recipe.scopes.is_empty() {
            return Err("benchmark workload recipe must declare at least one scope".into());
        }
        if self.recipe.root_templates.is_empty() || self.recipe.package_templates.is_empty() {
            return Err("benchmark workload recipe must declare root and package templates".into());
        }
        Ok(())
    }

    pub fn ensure_default_benchmark_contract(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.recipe.expected_contract.regular_file_count < 100_000 {
            return Err(
                "default benchmark workload contract must declare at least 100,000 regular files"
                    .into(),
            );
        }
        Ok(())
    }

    pub fn ensure_summary_matches_contract(
        &self,
        summary: &WorkloadSummary,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let contract = &self.recipe.expected_contract;
        if summary.package_count != contract.package_count {
            return Err(format!(
                "generated workload package count mismatch: expected {}, got {}",
                contract.package_count, summary.package_count
            )
            .into());
        }
        if summary.regular_file_count != contract.regular_file_count {
            return Err(format!(
                "generated workload file count mismatch: expected {}, got {}",
                contract.regular_file_count, summary.regular_file_count
            )
            .into());
        }
        if summary.directory_count < contract.min_directory_count {
            return Err(format!(
                "generated workload directory count below contract: expected at least {}, got {}",
                contract.min_directory_count, summary.directory_count
            )
            .into());
        }
        if summary.max_package_depth != contract.max_package_depth {
            return Err(format!(
                "generated workload depth mismatch: expected {}, got {}",
                contract.max_package_depth, summary.max_package_depth
            )
            .into());
        }
        Ok(())
    }

    fn root_package_names(&self) -> Vec<String> {
        (0..self.recipe.root_packages)
            .map(|index| self.package_name(index))
            .collect()
    }

    fn build_package_plan(&self) -> Vec<PackagePlan> {
        let mut packages = Vec::new();
        let mut next_index = 0usize;
        let mut current_level = Vec::new();

        for _ in 0..self.recipe.root_packages {
            let name = self.package_name(next_index);
            current_level.push(packages.len());
            packages.push(PackagePlan {
                index: next_index,
                depth: 1,
                safe_name: safe_name(&name),
                relative_dir: package_relative_dir(None, &name),
                name,
                dependency_names: Vec::new(),
            });
            next_index += 1;
        }

        for fanout in &self.recipe.fanout_per_depth {
            let mut next_level = Vec::new();
            for parent_index in &current_level {
                let parent_relative_dir = packages[*parent_index].relative_dir.clone();
                let parent_name = packages[*parent_index].name.clone();
                let child_depth = packages[*parent_index].depth + 1;
                let mut dependency_names = Vec::new();
                for _ in 0..*fanout {
                    let child_name = self.package_name(next_index);
                    dependency_names.push(child_name.clone());
                    next_level.push(packages.len());
                    packages.push(PackagePlan {
                        index: next_index,
                        depth: child_depth,
                        safe_name: safe_name(&child_name),
                        relative_dir: package_relative_dir(Some(&parent_relative_dir), &child_name),
                        name: child_name,
                        dependency_names: Vec::new(),
                    });
                    next_index += 1;
                }
                let _ = parent_name;
                packages[*parent_index].dependency_names = dependency_names;
            }
            current_level = next_level;
        }

        packages
    }

    fn package_name(&self, index: usize) -> String {
        let base = match index % 8 {
            0 => "runtime",
            1 => "helpers",
            2 => "schema",
            3 => "config",
            4 => "types",
            5 => "router",
            6 => "builder",
            _ => "plugin",
        };
        let package = format!("{base}-{:05}", index + 1);
        if self.recipe.scoped_package_period > 0
            && (index + 1) % self.recipe.scoped_package_period == 0
        {
            let scope = &self.recipe.scopes[index % self.recipe.scopes.len()];
            format!("{scope}/{package}")
        } else {
            package
        }
    }
}

fn summarize_generated_tree(
    input_root: &Path,
    workload: &BenchmarkWorkload,
) -> Result<WorkloadSummary, Box<dyn std::error::Error + Send + Sync>> {
    let mut regular_file_count = 0u64;
    let mut directory_count = 0u64;
    let mut total_bytes = 0u64;

    for entry in walkdir::WalkDir::new(input_root).follow_links(false) {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            regular_file_count += 1;
            total_bytes += metadata.len();
        } else if metadata.is_dir() {
            directory_count += 1;
        }
    }

    Ok(WorkloadSummary {
        name: workload.recipe.name.clone(),
        recipe_path: display_recipe_path(&workload.recipe_path),
        package_count: workload.recipe.expected_contract.package_count,
        regular_file_count,
        directory_count,
        total_bytes,
        max_package_depth: workload.recipe.expected_contract.max_package_depth,
        dominant_file_types: workload.recipe.expected_contract.dominant_file_types.clone(),
    })
}

fn load_templates(
    specs: &[TemplateSpec],
    recipe_dir: &Path,
) -> Result<Vec<LoadedTemplate>, Box<dyn std::error::Error + Send + Sync>> {
    specs.iter()
        .map(|spec| {
            let template_path = recipe_dir.join(&spec.template);
            Ok(LoadedTemplate {
                relative_path: spec.relative_path.clone(),
                contents: std::fs::read_to_string(&template_path).map_err(|e| {
                    format!(
                        "failed to read benchmark workload template {}: {e}",
                        template_path.display()
                    )
                })?,
            })
        })
        .collect()
}

fn package_relative_dir(parent_relative_dir: Option<&Path>, package_name: &str) -> PathBuf {
    let mut relative_dir = match parent_relative_dir {
        Some(parent_relative_dir) => parent_relative_dir.join("node_modules"),
        None => PathBuf::from("node_modules"),
    };
    for component in package_name.split('/') {
        relative_dir.push(component);
    }
    relative_dir
}

fn package_context(package: &PackagePlan) -> Vec<(&'static str, String)> {
    let dependency_list = if package.dependency_names.is_empty() {
        "- (leaf package)".to_string()
    } else {
        package
            .dependency_names
            .iter()
            .map(|name| format!("- `{name}`"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    vec![
        ("PACKAGE_NAME", package.name.clone()),
        ("PACKAGE_SAFE_NAME", package.safe_name.clone()),
        ("PACKAGE_VERSION", format!("1.{}.0", (package.index % 9) + 1)),
        ("PACKAGE_DEPTH", package.depth.to_string()),
        ("PACKAGE_INDEX", (package.index + 1).to_string()),
        ("DEPENDENCY_COUNT", package.dependency_names.len().to_string()),
        ("DEPENDENCY_LIST", dependency_list),
        (
            "DEPENDENCY_NAME_ARRAY",
            string_literal_array(&package.dependency_names),
        ),
        (
            "DEPENDENCIES_JSON",
            json_dependency_entries(&package.dependency_names, 4),
        ),
        ("IMPORT_BLOCK", import_block(&package.dependency_names)),
        (
            "SCHEMA_PROPERTIES",
            schema_properties(&package.dependency_names, 4),
        ),
    ]
}

fn render_template(template: &str, context: &[(&str, String)]) -> String {
    let mut rendered = template.to_string();
    for (key, value) in context {
        rendered = rendered.replace(&format!("{{{{{key}}}}}"), value);
    }
    rendered
}

fn safe_name(package_name: &str) -> String {
    package_name
        .replace('@', "")
        .replace('/', "-")
        .replace('.', "-")
}

fn write_file(
    path: &Path,
    contents: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, contents)?;
    Ok(())
}

fn string_literal_array(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

fn json_dependency_entries(values: &[String], indent: usize) -> String {
    if values.is_empty() {
        return String::new();
    }
    let padding = " ".repeat(indent);
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let comma = if index + 1 == values.len() { "" } else { "," };
            format!("{padding}\"{value}\": \"1.0.0\"{comma}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn import_block(values: &[String]) -> String {
    if values.is_empty() {
        return String::new();
    }
    values
        .iter()
        .enumerate()
        .map(|(index, value)| format!("import * as dep_{index} from \"{value}\";"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn schema_properties(values: &[String], indent: usize) -> String {
    if values.is_empty() {
        return String::new();
    }
    let padding = " ".repeat(indent);
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let comma = if index + 1 == values.len() { "" } else { "," };
            format!(
                "{padding}\"dependency_{index}\": {{ \"type\": \"string\", \"default\": \"{value}\" }}{comma}"
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Default)]
struct DirectoryTracker {
    directories: BTreeSet<PathBuf>,
}

impl DirectoryTracker {
    fn track_parent_of(&mut self, relative_path: &Path) {
        let mut current = relative_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        self.directories.insert(PathBuf::new());
        loop {
            self.directories.insert(current.clone());
            if current.as_os_str().is_empty() {
                break;
            }
            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => break,
            }
        }
    }

    fn len(&self) -> u64 {
        self.directories.len() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::{BenchmarkWorkload, TemplateSpec, WorkloadContract, WorkloadRecipe};
    use tempfile::TempDir;

    #[test]
    fn default_workload_recipe_meets_documented_contract() {
        let workload = BenchmarkWorkload::load_default().expect("load default workload");
        let summary = workload.planned_summary().expect("plan summary");

        assert_eq!(
            summary.recipe_path,
            "benches/workloads/node-modules-100k/recipe.json"
        );
        assert_eq!(summary.package_count, workload.expected_contract().package_count);
        assert_eq!(
            summary.regular_file_count,
            workload.expected_contract().regular_file_count
        );
        assert_eq!(
            summary.max_package_depth,
            workload.expected_contract().max_package_depth
        );
        assert!(
            summary.directory_count >= workload.expected_contract().min_directory_count,
            "directory count {} should meet minimum {}",
            summary.directory_count,
            workload.expected_contract().min_directory_count
        );
    }

    #[test]
    fn small_recipe_materializes_nested_workload() {
        let asset_dir = TempDir::new().expect("asset dir");
        let output_dir = TempDir::new().expect("output dir");
        std::fs::create_dir_all(asset_dir.path().join("templates")).expect("templates dir");
        std::fs::write(
            asset_dir.path().join("templates/root.tpl"),
            "{\"dependencies\": {\n{{ROOT_DEPENDENCIES_JSON}}\n}}\n",
        )
        .expect("root template");
        std::fs::write(
            asset_dir.path().join("templates/pkg.tpl"),
            "package={{PACKAGE_NAME}}\ndeps={{DEPENDENCY_COUNT}}\n",
        )
        .expect("package template");
        let recipe = WorkloadRecipe {
            name: "mini".to_string(),
            description: "mini recipe".to_string(),
            root_package_name: "mini-root".to_string(),
            root_packages: 2,
            fanout_per_depth: vec![2],
            scoped_package_period: 0,
            scopes: vec!["@mini".to_string()],
            root_templates: vec![TemplateSpec {
                relative_path: "package.json".into(),
                template: "templates/root.tpl".into(),
            }],
            package_templates: vec![TemplateSpec {
                relative_path: "package.txt".into(),
                template: "templates/pkg.tpl".into(),
            }],
            expected_contract: WorkloadContract {
                package_count: 6,
                regular_file_count: 7,
                max_package_depth: 2,
                min_directory_count: 4,
                dominant_file_types: vec!["text".to_string()],
            },
        };
        let recipe_path = asset_dir.path().join("recipe.json");
        std::fs::write(&recipe_path, serde_json::to_vec_pretty(&recipe).expect("json"))
            .expect("recipe");
        let workload = BenchmarkWorkload::from_recipe_path(&recipe_path).expect("workload");
        let summary = workload
            .materialize(output_dir.path().join("input").as_path())
            .expect("materialize");

        assert_eq!(summary.package_count, 6);
        assert_eq!(summary.regular_file_count, 7);
        assert!(
            output_dir
                .path()
                .join("input/node_modules/runtime-00001/node_modules/schema-00003/package.txt")
                .is_file()
        );
    }
}
