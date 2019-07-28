use std::{env, fmt, fs, io};
use std::fmt::Write;

type Result<T> = std::result::Result<T, std::boxed::Box<dyn std::error::Error>>;
type Table = toml::value::Table;
type UnitResult = Result<()>;

fn write_module(
    output: &mut dyn io::Write,
    module: Option<&str>,
    blocks: &[String],
) -> UnitResult {
    if let Some(module_name) = module {
        for submodule in module_name.split("::") {
            writeln!(output, "pub mod {} {{", submodule)?;
        }
    }
    for block in blocks {
        writeln!(output, "{}", block)?;
    }
    if let Some(module_name) = module {
        for _ in module_name.split("::") {
            writeln!(output, "}}")?;
        }
    }
    Ok(())
}

fn write_string_enum(
    output: &mut dyn fmt::Write,
    name: &str,
    meta: &Table,
    values: &Table,
) -> UnitResult {
    let x = output;
    let docs = meta["docs"].as_str().unwrap();
    let derives = "Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize";
    writeln!(x, "/// {}", docs)?;
    writeln!(x, "#[derive({})]", derives)?;
    writeln!(x, "pub enum {} {{", name)?;
    for (i, (name, value)) in values.iter().enumerate() {
        let docs = value["docs"].as_str().unwrap();
        writeln!(x, "  /// {}", docs)?;
        writeln!(x, "  {} = {},", name, i + 1)?;
    }
    writeln!(x, "}}")?;
    writeln!(x, "")?;
    writeln!(x, "impl<'de> ::serde::de::Deserialize<'de> for Checked<{}> {{", name)?;
    writeln!(x, "  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>")?;
    writeln!(x, "    where D: ::serde::de::Deserializer")?;
    writeln!(x, "  {{")?;
    writeln!(x, "    struct Visitor;")?;
    writeln!(x, "    impl<'de> ::serde::de::Visitor<'de> for Visitor {{")?;
    writeln!(x, "      type Value = Checked<{}>;", name)?;
    writeln!(x, "      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>")?;
    writeln!(x, "        where E: ::serde::de::Error")?;
    writeln!(x, "      {{")?;
    writeln!(x, "        Ok(")?;
    writeln!(x, "          match value {{")?;
    for (variant, value) in values {
        let str_value = value["value"].as_str().unwrap();
        writeln!(x, "            \"{}\" => Checked::Valid({}::{}),", str_value, name, variant)?;
    }
    writeln!(x, "            _ => Invalid,")?;
    writeln!(x, "          }}")?;
    writeln!(x, "        )")?;
    writeln!(x, "      }}")?;
    writeln!(x, "    }}")?;
    writeln!(x, "  }}")?;
    writeln!(x, "}}")?;
    writeln!(x, "")?;
    writeln!(x, "impl {} {{", name)?;
    writeln!(x, "  /// Returns the equivalent string value.")?;
    writeln!(x, "  pub fn as_str(&self) -> &'static str {{")?;
    writeln!(x, "    match *self {{")?;
    for (variant, value) in values {
        let str_value = value["value"].as_str().unwrap();
        writeln!(x, "      {}::{} => \"{}\",", name, variant, str_value)?;
    }
    writeln!(x, "    }}")?;
    writeln!(x, "  }}")?;
    writeln!(x, "}}")?;
    writeln!(x, "")?;
    writeln!(x, "impl ::serde::ser::Serialize for {} {{", name)?;
    writeln!(x, "  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>")?;
    writeln!(x, "    where S: ::serde::ser::Serializer")?;
    writeln!(x, "  {{")?;
    writeln!(x, "    serializer.serialize_str(self.as_str())")?;
    writeln!(x, "  }}")?;
    write!(x, "}}")?;
    Ok(())
}

fn write_integer_enum(
    output: &mut dyn fmt::Write,
    name: &str,
    meta: &Table,
    values: &Table,
) -> UnitResult {
    let x = output;
    let docs = meta["docs"].as_str().unwrap();
    let derives = "Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize";
    writeln!(x, "/// {}", docs)?;
    writeln!(x, "#[derive({})]", derives)?;
    writeln!(x, "pub enum {} {{", name)?;
    for (i, (name, value)) in values.iter().enumerate() {
        let docs = value["docs"].as_str().unwrap();
        writeln!(x, "  /// {}", docs)?;
        writeln!(x, "  {} = {},", name, i + 1)?;
    }
    writeln!(x, "}}")?;
    writeln!(x, "")?;
    writeln!(x, "impl<'de> ::serde::de::Deserialize<'de> for Checked<{}> {{", name)?;
    writeln!(x, "  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>")?;
    writeln!(x, "    where D: ::serde::de::Deserializer")?;
    writeln!(x, "  {{")?;
    writeln!(x, "    struct Visitor;")?;
    writeln!(x, "    impl<'de> ::serde::de::Visitor<'de> for Visitor {{")?;
    writeln!(x, "      type Value = Checked<{}>;", name)?;
    writeln!(x, "      fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>")?;
    writeln!(x, "        where E: ::serde::de::Error")?;
    writeln!(x, "      {{")?;
    writeln!(x, "        Ok(")?;
    writeln!(x, "          match value as u32 {{")?;
    for (variant, value) in values {
        let int_value = value["value"].as_integer().unwrap();
        writeln!(x, "            {} => Checked::Valid({}::{}),", int_value, name, variant)?;
    }
    writeln!(x, "            _ => Invalid,")?;
    writeln!(x, "          }}")?;
    writeln!(x, "        )")?;
    writeln!(x, "      }}")?;
    writeln!(x, "    }}")?;
    writeln!(x, "  }}")?;
    writeln!(x, "}}")?;
    writeln!(x, "")?;
    writeln!(x, "impl {} {{", name)?;
    writeln!(x, "  /// Returns the equivalent GLenum value.")?;
    writeln!(x, "  pub fn as_gl_enum(&self) -> u32 {{")?;
    writeln!(x, "    match *self {{")?;
    for (variant, value) in values {
        let int_value = value["value"].as_integer().unwrap();
        writeln!(x, "      {}::{} => {},", name, variant, int_value)?;
    }
    writeln!(x, "    }}")?;
    writeln!(x, "  }}")?;
    writeln!(x, "}}")?;
    writeln!(x, "")?;
    writeln!(x, "impl ::serde::ser::Serialize for {} {{", name)?;
    writeln!(x, "  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>")?;
    writeln!(x, "    where S: ::serde::ser::Serializer")?;
    writeln!(x, "  {{")?;
    writeln!(x, "    serializer.serialize_u32(self.as_gl_enum())")?;
    writeln!(x, "  }}")?;
    write!(x, "}}")?;
    Ok(())
}

fn write_struct(
    output: &mut dyn fmt::Write,
    name: &str,
    meta: &Table,
    fields: &Table,
) -> UnitResult {
    let mut extra = String::new();

    let docs = meta["docs"].as_str().unwrap();
    writeln!(output, "/// {}", docs)?;
    writeln!(output, "#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]")?;
    writeln!(output, "pub struct {} {{", name)?;
    for (name, field) in fields {
        let docs = field["docs"].as_str().unwrap();
        let ty = field["ty"].as_str().unwrap();
        let optional = field.get("optional").map(|value| value.as_bool().unwrap()).unwrap_or(false);
        let default = field.get("default");
        if optional {
            assert!(default.is_none());
        }

        writeln!(output, "  /// {}", docs)?;
        if optional {
            writeln!(output, "  #[serde(default, skip_serializing_if = \"Option::is_none\")]")?;
        } else if default.is_some() {
            writeln!(output, "  #[serde(default = \"{}_default\")]", name)?;
            writeln!(output, "  #[serde(skip_serializing_if = \"{}_is_default\")]", name)?;
        }
        match ty {
            // Data types that support optional semantics:
            "String" if optional => {
                writeln!(output, "  pub {}: Option<String>,", name)?;
            },
            "String" => {
                writeln!(output, "  pub {}: String,", name)?;
            },
            "Integer" if optional => {
                writeln!(output, "  pub {}: Option<u32>,", name)?;
            },
            "Integer" => {
                writeln!(output, "  pub {}: u32,", name)?;
                if let Some(value) = default {
                    let default_u32 = value.as_integer().unwrap();
                    writeln!(extra, "fn {}_default() -> u32 {{ {} }}", name, default_u32)?;
                    writeln!(extra, "fn {}_is_default(x: u32) -> u32 {{ x == {} }}", name, default_u32)?;
                }
            },
            "Index" if optional => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "  pub {}: Option<Index<::{}>>,", name, of)?;
            },
            "Index" => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "  pub {}: Index<::{}>,", name, of)?;
            },
            "Struct" if optional => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "  pub {}: Option<::{}>,", name, of)?;
            },
            "Struct" => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "  pub {}: ::{},", name, of)?;
            },
            "Float" if optional => {
                writeln!(output, "  pub {}: Option<f32>,", name)?;
            },
            "Float" => {
                writeln!(output, "  pub {}: f32,", name)?;
                if let Some(value) = default {
                    let default_f32 = value.as_float().unwrap();
                    writeln!(extra, "fn {}_default() -> f32 {{ {} }}", name, default_f32)?;
                    writeln!(extra, "fn {}_is_default(x: f32) -> f32 {{ approx_eq!(x, {}) }}", name, default_f32)?;
                }
            },
            // Data types that don't support optional semantics:
            "Bool" => {
                writeln!(output, "  pub {}: bool,", name)?;
                if let Some(value) = default {
                    let default_bool = value.as_bool().unwrap();
                    writeln!(extra, "fn {}_default() -> bool {{ {} }}", name, default_bool)?;
                    writeln!(extra, "fn {}_is_default(x: bool) -> bool {{ x == {} }}", name, default_bool)?;
                }
            },
            "Enum" => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "  pub {}: Checked<::{}>,", name, of)?;
            },
            "Any" => {
                writeln!(output, "  pub {}: Option<::std::boxed::Box<::serde::value::RawValue>>,", name)?;
            },
            unknown => panic!("unknown type '{}'", unknown),
        }
    }

    if extra.is_empty() {
        write!(output, "}}")?;
    } else {
        writeln!(output, "}}")?;
        write!(output, "{}", extra)?;
    }

    Ok(())
}

fn write_struct_accessor(
    output: &mut dyn fmt::Write,
    name: &str,
    meta: &Table,
    fields: &Table,
) -> UnitResult {
    let docs = meta["docs"].as_str().unwrap();
    writeln!(output, "/// {}", docs)?;
    writeln!(output, "#[derive(Clone, Debug)]")?;
    writeln!(output, "pub struct {}<'a> {{", name)?;
    writeln!(output, "  pub(crate) document: &'a ::Document,")?;
    writeln!(output, "  pub(crate) json: &'a ::json::{},", name)?;
    writeln!(output, "}}")?;
    writeln!(output, "")?;
    writeln!(output, "impl<'a> {}<'a> {{", name)?;
    for (name, field) in fields {
        let docs = field["docs"].as_str().unwrap();
        let optional = field.get("optional").map(|value| value.as_bool().unwrap()).unwrap_or(false);

        writeln!(output, "  /// {}", docs)?;
        match field["ty"].as_str().unwrap() {
            // Data types that support optional semantics:
            "Index" if optional => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "  pub fn {}(&self) -> Option<::{}<'a>> {{", name, of)?;
                writeln!(output, "    self.{}.as_ref().map(|index| self.document.get(index))", name)?;
            },
            "Index" => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "  pub fn {}(&self) -> ::{}<'a> {{", name, of)?;
                writeln!(output, "    self.document.get(&self.{})", name)?;
            },
            "Struct" if optional => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "  pub fn {}(&self) -> Option<::{}<'a>> {{", name, of)?;
                writeln!(output, "    self.{}.as_ref().map(|json| ::{}::new(self.document, json))", name, of)?;
            },
            "Struct" => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "  pub fn {}(&self) -> ::{}<'a> {{", name, of)?;
                writeln!(output, "    ::{}::new(self.document, &self.{})", of, name)?;
            },
            "String" if optional => {
                writeln!(output, "  pub fn {}(&self) -> Option<&'a str> {{", name)?;
                writeln!(output, "    self.{}.as_ref().map(|string| string.as_str())", name)?;
            },
            "String" => {
                writeln!(output, "  pub fn {}(&self) -> &'a str {{", name)?;
                writeln!(output, "    self.{}.as_str()", name)?;
            },
            "Integer" if optional => {
                writeln!(output, "  pub fn {}(&self) -> Option<u32> {{", name)?;
                writeln!(output, "    self.{}.clone()", name)?;
            },
            "Integer" => {
                writeln!(output, "  pub fn {}(&self) -> u32 {{", name)?;
                writeln!(output, "    self.{}", name)?;
            },
            "Float" if optional => {
                writeln!(output, "  pub fn {}(&self) -> Option<f32> {{", name)?;
                writeln!(output, "    self.{}.clone()", name)?;
            },
            "Float" => {
                writeln!(output, "  pub fn {}(&self) -> f32 {{", name)?;
                writeln!(output, "    self.{}", name)?;
            },
            // Data types that don't support optional semantics:
            "Bool" => {
                writeln!(output, "  pub fn {}(&self) -> bool {{", name)?;
                writeln!(output, "    self.{}", name)?;
            },
            "Enum" => {
                let of = field["of"].as_str().unwrap();
                writeln!(output, "  pub fn {}(&self) -> ::{} {{", name, of)?;
                writeln!(output, "    self.{}.unwrap()", name)?;
            },
            "Any" => {
                writeln!(output, "  pub fn {}(&self) -> Option<&::serde::value::RawValue> {{", name)?;
                writeln!(output, "    self.{}.as_ref().map(|boxed| &*boxed)", name)?;
            },
            unknown => panic!("unknown type '{}'", unknown),
        };
        writeln!(output, "  }}")?;
    }
    write!(output, "}}")?;
    Ok(())
}

fn run() -> UnitResult {
    let path = env::args().nth(1).expect("file path");
    let file_content = fs::read_to_string(&path)?;
    let value: toml::Value = toml::from_str(&file_content)?;
    let meta = value["meta"].as_table().unwrap();
    let name = meta["ident"].as_str().unwrap();
    let module = meta["module"].as_str(); // note: may be nested e.g. foo::bar
    let _qpath = module.map_or_else(|| name.to_string(), |x| format!("{}::{}", x, name));
    let kind = meta["kind"].as_str().unwrap();
    let mut blocks = vec![];
    match kind {
        "Struct" => {
            let mut block = String::new();
            let fields = value["fields"].as_table().unwrap();
            write_struct(&mut block, &name, meta, fields)?;
            blocks.push(block);

            block = String::new();
            write_struct_accessor(&mut block, &name, meta, fields)?;
            blocks.push(block);
        },
        "Enum" => {
            let of = meta["of"].as_str().unwrap();
            let values = value["values"].as_table().unwrap();
            let mut block = String::new();
            match of {
                "String" => write_string_enum(&mut block, &name, &meta, &values)?,
                "Integer" => write_integer_enum(&mut block, &name, &meta, &values)?,
                _ => panic!("unknown enum encoding"),
            }
            blocks.push(block);
        }
        _ => panic!("unknown data kind"),
    }
    let mut output = io::stdout();
    write_module(&mut output, module, &blocks)?;
    writeln!(&mut output as &mut dyn io::Write, "")?;
    Ok(())
}

fn main() {
    run().expect("runtime error");
}
