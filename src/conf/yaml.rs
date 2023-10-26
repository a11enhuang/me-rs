use std::collections::HashMap;

use serde_yaml::{Mapping, Value};

use crate::conf::PropertyLoader;

pub struct YamlPropertyLoader {}

impl PropertyLoader for YamlPropertyLoader {
    fn load(&self, conf: String) -> Result<HashMap<String, String>, String> {
        let source: Mapping = serde_yaml::from_str(&conf).map_err(|e| { e.to_string() })?;
        let result = self.get_flattened_map(source);
        Ok(result)
    }

    fn get_file_extensions(&self) -> Vec<String> {
        return vec![String::from("yaml"), String::from("yml")];
    }
}

impl YamlPropertyLoader {
    pub fn new() -> YamlPropertyLoader {
        return YamlPropertyLoader {};
    }
    fn get_flattened_map(&self, source: Mapping) -> HashMap<String, String> {
        let mut result = HashMap::new();
        self.build_flattened_map(&mut result, &source, None);
        return result;
    }

    /// 将Yaml类型的数据展开平铺
    fn build_flattened_map(&self, result: &mut HashMap<String, String>, source: &Mapping, path: Option<&String>) {
        for k in source.keys() {
            let mut key = String::from(k.as_str().unwrap());
            let o = source.get(&key);
            if o.is_none() {
                return;
            }
            let value = o.unwrap();
            if path.is_some() {
                let mut path = path.cloned().unwrap();
                if key.starts_with("[") {
                    key.insert_str(0, &path)
                } else {
                    path.push_str(".");
                    key.insert_str(0, &path)
                }
            }
            if value.is_string() {
                let str = value.as_str().unwrap_or("");
                result.insert(key, String::from(str));
            } else if value.is_mapping() {
                let mapping = value.as_mapping().unwrap();
                self.build_flattened_map(result, mapping, Some(&key))
            } else if value.is_sequence() {
                let connection = value.as_sequence();
                if connection.is_none() {
                    result.insert(key, String::from(""));
                    continue;
                }
                let arr = connection.unwrap();
                if arr.len() == 0 {
                    result.insert(key, String::from(""));
                    continue;
                }
                let mut count = 0;
                for value in arr {
                    let mut m = Mapping::new();
                    m.insert(Value::from(format!("[{}]", count)), value.clone());
                    self.build_flattened_map(result, &m, Some(&key));
                    count += 1;
                }
            } else {
                result.insert(key, String::from(""));
            }
        }
    }
}