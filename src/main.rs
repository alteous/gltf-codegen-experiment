use std::{fs, io};

type Result<T> = std::result::Result<T, std::boxed::Box<dyn std::error::Error>>;
type Table = toml::value::Table;
type UnitResult = Result<()>;

fn newline(output: &mut dyn io::Write) -> UnitResult {
    writeln!(output, "")?;
    Ok(())
}

fn write_string_enum(
    output: &mut dyn io::Write,
    name: &str,
    meta: &Table,
    values: &Table,
) -> UnitResult {
    let docs = meta["docs"].as_str().unwrap();
    let derives = "Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize";
    writeln!(output, "/// {}", docs)?;
    writeln!(output, "#[derive({})]", derives)?;
    writeln!(output, "enum {} {{", name)?;
    for (name, value) in values {
        let docs = value["docs"].as_str().unwrap();
        writeln!(output, "    /// {}", docs)?;
        writeln!(output, "    {},", name)?;
    }
    writeln!(output, "}}")?;
    Ok(())
}

fn write_integer_enum(
    output: &mut dyn io::Write,
    name: &str,
    meta: &Table,
    values: &Table,
) -> UnitResult {
    let docs = meta["docs"].as_str().unwrap();
    let derives = "Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize";
    writeln!(output, "/// {}", docs)?;
    writeln!(output, "#[derive({})]", derives)?;
    writeln!(output, "enum {} {{", name)?;
    for (name, value) in values {
        let docs = value["docs"].as_str().unwrap();
        let int_value = value["value"].as_integer().unwrap();
        writeln!(output, "    /// {}", docs)?;
        writeln!(output, "    {} = {},", name, int_value)?;
    }
    writeln!(output, "}}")?;
    Ok(())
}

fn write_struct(
    output: &mut dyn io::Write,
    name: &str,
    meta: &Table,
    fields: &Table,
) -> UnitResult {
    let docs = meta["docs"].as_str().unwrap();
    writeln!(output, "/// {}", docs)?;
    writeln!(output, "#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]")?;
    writeln!(output, "struct {} {{", name)?;
    for (name, field) in fields {
        let docs = field["docs"].as_str().unwrap();
        let ty = match field["ty"].as_str().unwrap() {
            "Index" => {
                let of = field["of"].as_str().unwrap();
                format!("Index<{}>", of)
            },
            "String" => "String".to_string(),
            "Enum" => {
                let of = field["of"].as_str().unwrap();
                format!("Checked<{}>", of)
            },
            _ => panic!("unknown type"),
        };
        writeln!(output, "    /// {}", docs)?;
        writeln!(output, "    pub {}: {},", name, ty)?;
    }
    writeln!(output, "}}")?;
    Ok(())
}

fn run() -> UnitResult {
    let toml_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/toml");
    let mut output = io::stdout();
    for result in fs::read_dir(toml_dir)? {
        let entry = result?;
        let path = entry.path();
        let fully_qualified_name = path.file_stem().unwrap().to_str().unwrap();
        let name = fully_qualified_name.split("::").last().unwrap();
        let file_content = fs::read_to_string(&path)?;
        let value: toml::Value = toml::from_str(&file_content)?;
        let meta = value["meta"].as_table().unwrap();
        let kind = meta["kind"].as_str().unwrap();
        match kind {
            "struct" => {
                let fields = value["fields"].as_table().unwrap();
                write_struct(&mut output, &name, meta, fields)?;
            },
            "enum" => {
                let ty = meta["ty"].as_str().unwrap();
                let values = value["values"].as_table().unwrap();
                match ty {
                    "string" => write_string_enum(&mut output, &name, &meta, &values)?,
                    "integer" => write_integer_enum(&mut output, &name, &meta, &values)?,
                    _ => panic!("unknown enum encoding"),
                }
            }
            _ => panic!("unknown data kind"),
        }

        newline(&mut output)?;
    }
    Ok(())
}

fn main() {
    run().expect("runtime error");
}
