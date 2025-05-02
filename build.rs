use std::{env, fs, path::Path};

fn main() {
    /*let app_root = match env::var("CORROSIVE_APP_ROOT") {
        Ok(val) => val,
        Err(_) => {
            println!("cargo:warning=CORROSIVE_APP_ROOT not set. Skipping check.");
            return;
        }
    };

    let app_path = Path::new(&app_root);
    let app_cargo_path = app_path.join("Cargo.toml");

    if !app_cargo_path.exists() {
        println!("cargo:warning=CORROSIVE_APP_ROOT does not point to a valid Cargo.toml");
        return;
    }

    let our_deps = extract_dependencies("Cargo.toml");
    let app_deps = extract_dependencies(app_cargo_path.to_str().unwrap());

    for (name, version) in our_deps.iter() {
        if let Some(app_version) = app_deps.get(name) {
            if app_version != version {
                panic!(
                    "Dependency mismatch for `{}`: expected `{}`, found `{}` in app",
                    name, version, app_version
                );
            }
        } else {
            panic!("Dependency `{}` not found in app at CORROSIVE_APP_ROOT", name);
        }
    }*/
}

/*fn extract_dependencies(path: &str) -> std::collections::HashMap<String, String> {
    let content = fs::read_to_string(path).expect("Failed to read Cargo.toml");
    let cargo_toml: toml::Value = content.parse().expect("Invalid Cargo.toml");

    let mut deps = std::collections::HashMap::new();

    if let Some(dependencies) = cargo_toml.get("dependencies") {
        if let Some(dependencies) = dependencies.as_table() {
            for (name, value) in dependencies.iter() {
                if let Some(version) = value.as_str() {
                    deps.insert(name.clone(), version.to_string());
                } else if let Some(table) = value.as_table() {
                    if let Some(version) = table.get("version").and_then(|v| v.as_str()) {
                        deps.insert(name.clone(), version.to_string());
                    }
                }
            }
        }
    }

    deps
}*/
