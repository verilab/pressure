use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};
use yaml_rust::Yaml;

use crate::Entry;

#[derive(Debug)]
struct SerYaml<'a>(&'a Yaml);

impl<'a> Serialize for SerYaml<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            Yaml::Real(v) => serializer.serialize_f64(v.parse::<f64>().unwrap()),
            Yaml::Integer(v) => serializer.serialize_i64(*v),
            Yaml::String(v) => serializer.serialize_str(v),
            Yaml::Boolean(v) => serializer.serialize_bool(*v),
            Yaml::Array(v) => {
                let mut s = serializer.serialize_seq(Some(v.len()))?;
                for elem in v.iter() {
                    s.serialize_element(&SerYaml(elem))?;
                }
                s.end()
            }
            Yaml::Hash(v) => {
                let mut s = serializer.serialize_map(Some(v.len()))?;
                for (key, val) in v.iter() {
                    s.serialize_entry(&SerYaml(key), &SerYaml(val))?;
                }
                s.end()
            }
            Yaml::Alias(v) => serializer.serialize_u64(*v as u64),
            Yaml::Null => serializer.serialize_none(),
            _ => serializer.serialize_none(),
        }
    }
}

impl Serialize for Entry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_map(None)?;
        for (k, v) in self.meta.as_hash().unwrap() {
            s.serialize_entry(&SerYaml(k), &SerYaml(v))?;
        }
        s.serialize_entry("url", "")?;
        s.serialize_entry("filepath", &self.filepath)?;
        s.serialize_entry("meta", &SerYaml(&self.meta))?;
        s.serialize_entry("content", &self.content)?;
        s.end()
    }
}
