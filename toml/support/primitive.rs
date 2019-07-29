// support/primitive.rs

use accessor::Accessor;
use serde::{ser, de};
use std::collections::hash_map;

/// An `Iterator` that visits the attributes of a `Primitive`.
#[derive(Clone, Debug)]
pub struct Attributes<'a> {
    /// The parent `Document` struct.
    pub(crate) document: &'a Document,

    /// The internal attribute iterator.
    pub(crate) iter: hash_map::Iter<'a, Checked<Semantic>, Index<Accessor>>,
}

impl<'a> ExactSizeIterator for Attributes<'a> {}
impl<'a> Iterator for Attributes<'a> {
    type Item = Attribute<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(key, index)| {
                let semantic = key.as_ref().unwrap().clone();
                let accessor = self.document.accessors().nth(index.value()).unwrap();
                (semantic, accessor)
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> Primitive<'a> {
    /// Returns an `Iterator` that visits the vertex attributes.
    pub fn attributes(&self) -> Attributes {
        Attributes {
            document: self.mesh.document,
            iter: self.json.attributes.iter(),
        }
    }
}

/// Vertex attribute semantic name.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Semantic {
    /// Extra attribute name.
    #[cfg(feature = "extras")]
    Extras(String),

    /// XYZ vertex positions.
    Positions,

    /// XYZ vertex normals.
    Normals,

    /// XYZW vertex tangents where the `w` component is a sign value indicating the
    /// handedness of the tangent basis.
    Tangents,

    /// RGB or RGBA vertex color.
    Colors(u32),

    /// UV texture co-ordinates.
    TexCoords(u32),

    /// Joint indices.
    Joints(u32),

    /// Joint weights.
    Weights(u32),
}

impl Semantic {
    fn checked(s: &str) -> Checked<Self> {
        use self::Semantic::*;
        use crate::validation::Checked::*;
        match s {
            "NORMAL" => Valid(Normals),
            "POSITION" => Valid(Positions),
            "TANGENT" => Valid(Tangents),
            #[cfg(feature = "extras")]
            _ if s.starts_with("_") => Valid(Extras(s[1..].to_string())),
            _ if s.starts_with("COLOR_") => {
                match s["COLOR_".len()..].parse() {
                    Ok(set) => Valid(Colors(set)),
                    Err(_) => Invalid,
                }
            },
            _ if s.starts_with("TEXCOORD_") => {
                match s["TEXCOORD_".len()..].parse() {
                    Ok(set) => Valid(TexCoords(set)),
                    Err(_) => Invalid,
                }
            },
            _ if s.starts_with("JOINTS_") => {
                match s["JOINTS_".len()..].parse() {
                    Ok(set) => Valid(Joints(set)),
                    Err(_) => Invalid,
                }
            },
            _ if s.starts_with("WEIGHTS_") => {
                match s["WEIGHTS_".len()..].parse() {
                    Ok(set) => Valid(Weights(set)),
                    Err(_) => Invalid,
                }
            },
            _ => Invalid,
        }
    }
}

impl ser::Serialize for Semantic {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl ToString for Semantic {
    fn to_string(&self) -> String {
        use self::Semantic::*;
        match *self {
            Positions => "POSITION".into(),
            Normals => "NORMAL".into(),
            Tangents => "TANGENT".into(),
            Colors(set) => format!("COLOR_{}", set),
            TexCoords(set) => format!("TEXCOORD_{}", set),
            Joints(set) => format!("JOINTS_{}", set),
            Weights(set) => format!("WEIGHTS_{}", set),
            #[cfg(feature = "extras")]
            Extras(ref name) => format!("_{}", name),
        }
    }
}

impl ToString for Checked<Semantic> {
    fn to_string(&self) -> String {
        match *self {
            Checked::Valid(ref semantic) => semantic.to_string(),
            Checked::Invalid => "<invalid semantic name>".into(),
        }
    }
}

impl<'de> de::Deserialize<'de> for Checked<Semantic> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Checked<Semantic>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "semantic name")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where E: de::Error
            {
                Ok(Semantic::checked(value))
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}
