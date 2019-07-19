use inflections::Inflect;
use std::fs;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, std::boxed::Box<dyn std::error::Error>>;
type UnitResult = Result<()>;

fn run() -> UnitResult {
    let schema_dir = "/home/alteous/glTF/specification/2.0/schema/";
    let mut names = vec![];
    for result in fs::read_dir(schema_dir)? {
        let entry = result?;
        let path = entry.path();
        let stem = path.file_stem().unwrap();
        let name = stem
            .to_str()
            .unwrap()
            .split(".schema")
            .next()
            .unwrap()
            .to_string()
            .replace(".", "-")
            .to_pascal_case();
        if !name.starts_with("GlTf") && name != "Extension" && name != "Extras" {
            names.push((name, path.to_path_buf()));
        }
    }
    names.sort_by_key(|x| x.0.clone()); // TODO: optimize.
    let mut group = std::collections::HashMap::<String, Vec<(String, PathBuf)>>::new();
    {
        let mut iter = names.into_iter();
        let mut next_group = vec![iter.next().unwrap()];
        for (name, path) in iter {
            if name.starts_with(&next_group[0].0) {
                let x = name.trim_start_matches(&next_group[0].0);
                next_group.push((x.to_string(), path));
            } else {
                group.insert(
                    next_group[0].0.clone(),
                    std::mem::replace(&mut next_group, vec![(name, path)]),
                );
            }
        }
        group.insert(next_group[0].0.clone(), next_group);
    }
    for (key, values) in group {
        println!("mod {} {{", key.to_snake_case());
        for (entry, path) in values {
            println!("    {} ({:?})", entry, path.file_name().unwrap());
        }
        println!("}}");
    }
    Ok(())
}

fn main() {
    run().expect("runtime error");
}
