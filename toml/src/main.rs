use std::{env, fs, io};

use std::fmt::Write;

type Result<T> = std::result::Result<T, std::boxed::Box<dyn std::error::Error>>;
type Table = toml::value::Table;
type UnitResult = Result<()>;

fn newline(output: &mut dyn io::Write) -> UnitResult {
    writeln!(output, "")?;
    Ok(())
}

fn write_string_enum(
    output: &mut dyn io::Write,
    module: Option<&str>,
    name: &str,
    meta: &Table,
    values: &Table,
) -> UnitResult {
    if let Some(module_name) = module {
        writeln!(output, "pub mod {} {{", module_name)?;
    }

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

    if module.is_some() {
        writeln!(output, "}}")?;
    }
    Ok(())
}

fn write_integer_enum(
    output: &mut dyn io::Write,
    module: Option<&str>,
    name: &str,
    meta: &Table,
    values: &Table,
) -> UnitResult {
    if let Some(module_name) = module {
        writeln!(output, "pub mod {} {{", module_name)?;
    }

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
    if module.is_some() {
        writeln!(output, "}}")?;
    }
    Ok(())
}

fn write_struct(
    output: &mut dyn io::Write,
    module: Option<&str>,
    name: &str,
    meta: &Table,
    fields: &Table,
) -> UnitResult {
    let mut extra = String::new();
    
    writeln!(output, "pub mod json {{")?;
    if let Some(module_name) = module {
        writeln!(output, "pub mod {} {{", module_name)?;
    }

    let docs = meta["docs"].as_str().unwrap();
    writeln!(output, "/// {}", docs)?;
    writeln!(output, "#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]")?;
    writeln!(output, "pub struct {} {{", name)?;
    for (name, field) in fields {
        let docs = field["docs"].as_str().unwrap();
        writeln!(output, "    /// {}", docs)?;
        let ty = field["ty"].as_str().unwrap();
        match ty {
            "String" => {
                writeln!(output, "    pub {}: String,", name)?;
            },
            "Integer" => {
                if let Some(default) = field["default"].as_str() {
                    writeln!(output, "    #[serde(default = \"{}_default\")]", name)?;
                    writeln!(output, "    #[serde(skip_serializing_if = \"{}_is_default\")]", name)?;
                    writeln!(extra, "fn {}_default() -> u32 {{ {} }}", name, default)?;
                    writeln!(extra, "fn {}_is_default(x: u32) -> u32 {{ x == {} }}", name, default)?;
                }
                writeln!(output, "    pub {}: u32,", name)?;
            },
            "Bool" => {
                if let Some(default) = field["default"].as_str() {
                    writeln!(output, "    #[serde(default = \"{}_default\")]", name)?;
                    writeln!(output, "    #[serde(skip_serializing_if = \"{}_is_default\")]", name)?;
                    writeln!(extra, "fn {}_default() -> bool {{ {} }}", name, default)?;
                    writeln!(extra, "fn {}_is_default(x: bool) -> bool {{ x == {} }}", name, default)?;
                }
                writeln!(output, "    pub {}: bool,", name)?;
            },
            "Index" => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "    pub {}: Index<{}>,", name, of)?;
            },
            "Option" => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "    #[serde(default, skip_serializing_if = \"Option::is_none\")]")?;
                writeln!(output, "    pub {}: Option<{}>,", name, of)?;
            },
            "Enum" => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "    pub {}: Checked<{}>,", name, of)?;
            },
            _ => panic!("unknown type"),
        }
    }
    writeln!(output, "}}")?;
    writeln!(output, "{}", extra)?;

    if module.is_some() {
        writeln!(output, "}}")?;
    }
    writeln!(output, "}}")?;

    Ok(())
}

fn write_struct_accessor(
    output: &mut dyn io::Write,
    module: Option<&str>,
    name: &str,
    meta: &Table,
    fields: &Table,
) -> UnitResult {
    if let Some(module_name) = module {
        writeln!(output, "pub mod {} {{", module_name)?;
    }

    let docs = meta["docs"].as_str().unwrap();
    writeln!(output, "/// {}", docs)?;
    writeln!(output, "#[derive(Clone, Debug)]")?;
    writeln!(output, "pub struct {}<'a> {{", name)?;
    writeln!(output, "    pub(crate) document: &'a Document,")?;
    writeln!(output, "    pub(crate) json: &'a json::{},", name)?;
    writeln!(output, "}}")?;
    newline(output)?;

    writeln!(output, "impl<'a> {}<'a> {{", name)?;
    for (name, field) in fields {
        let docs = field["docs"].as_str().unwrap();
        writeln!(output, "    /// {}", docs)?;
        match field["ty"].as_str().unwrap() {
            "Index" => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "    pub fn {}(&self) -> {}<'a> {{", name, of)?;
                writeln!(output, "        self.document.get(&self.{})", name)?;
                writeln!(output, "    }}")?;
            },
            "Option" => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "    pub fn {}(&self) -> Option<&'a {}> {{", name, of)?;
                writeln!(output, "        self.{}.as_ref()", name)?;
                writeln!(output, "    }}")?;
            },
            "String" => {
                writeln!(output, "    pub fn {}(&self) -> &'a str {{", name)?;
                writeln!(output, "        self.{}.as_str()", name)?;
                writeln!(output, "    }}")?;
            },
            "Bool" => {
                writeln!(output, "    pub fn {}(&self) -> bool {{", name)?;
                writeln!(output, "        self.{}", name)?;
                writeln!(output, "    }}")?;
            },
            "Enum" => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "    pub fn {}(&self) -> {} {{", name, of)?;
                writeln!(output, "        self.{}.unwrap()", name)?;
                writeln!(output, "    }}")?;
            },
            "Integer" => {
                writeln!(output, "    pub fn {}(&self) -> u32 {{", name)?;
                writeln!(output, "        self.{}", name)?;
                writeln!(output, "    }}")?;
            },
            _ => panic!("unknown type"),
        };
    }
    writeln!(output, "}}")?;
    if module.is_some() {
        writeln!(output, "}}")?;
    }

    Ok(())
}

fn run() -> UnitResult {
    let path = env::args().nth(1).expect("file path");
    let file_content = fs::read_to_string(&path)?;
    let mut output = io::stdout();
    let value: toml::Value = toml::from_str(&file_content)?;
    let meta = value["meta"].as_table().unwrap();
    let id = meta["id"].as_str().unwrap();
    let mut segments = id.split("::");
    let mut name = segments.next().unwrap();
    let mut module = None;
    if let Some(x) = segments.next() {
        module = Some(name);
        name = x;
    }
    let kind = meta["kind"].as_str().unwrap();
    match kind {
        "Struct" => {
            let fields = value["fields"].as_table().unwrap();
            write_struct(&mut output, module, &name, meta, fields)?;
            write_struct_accessor(&mut output, module, &name, meta, fields)?;
        },
        "Enum" => {
            let ty = meta["ty"].as_str().unwrap();
            let values = value["values"].as_table().unwrap();
            match ty {
                "String" => write_string_enum(&mut output, module, &name, &meta, &values)?,
                "Integer" => write_integer_enum(&mut output, module, &name, &meta, &values)?,
                _ => panic!("unknown enum encoding"),
            }
        }
        _ => panic!("unknown data kind"),
    }
    newline(&mut output)?;
    Ok(())
}

fn main() {
    run().expect("runtime error");
}
