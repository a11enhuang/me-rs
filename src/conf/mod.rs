use std::{env, fs};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::conf::yaml::YamlPropertyLoader;

mod yaml;


/// 环境变量
pub struct Environment {
    property_loaders: HashMap<String, Rc<dyn PropertyLoader>>,
    properties: HashMap<String, String>,
}

impl Environment {
    /// 创建一个环境变量实例
    pub fn new() -> Environment {
        let mut env = Environment {
            property_loaders: HashMap::new(),
            properties: HashMap::new(),
        };
        env.register_property_loader(Rc::new(YamlPropertyLoader::new()));
        env.parse_command_args();
        return env;
    }

    pub fn register_property_loader(&mut self, loader: Rc<dyn PropertyLoader>) {
        let file_extensions: Vec<String> = loader.get_file_extensions();
        for k in file_extensions {
            self.property_loaders.insert(k, Rc::clone(&loader));
        }
    }

    fn parse_command_args(&mut self) {
        let args: Vec<String> = env::args().collect();
        for (_, arg) in args.iter().enumerate() {
            if arg.starts_with("--") && arg.contains("=") {
                let parsed: Vec<&str> = arg.split("=").collect();
                let key = String::from(*parsed.get(0).unwrap())
                    .strip_prefix("--")
                    .map(String::from)
                    .unwrap();
                let value = String::from(*parsed.get(1).unwrap());
                self.properties.insert(key, value);
            }
        }
    }
}


/// 解析配置
impl Environment {
    pub fn resolve_placeholders(&self, text: &str) -> String {
        return self.get_property(text).unwrap_or_else(|| String::from(text));
    }

    pub fn get_properties(&self, text: &str) -> Vec<String> {
        let key = self.prepare_key(text);
        let mut result: Vec<String> = Vec::new();
        for i in 0.. {
            let name = format!("{key}[{i}]");
            let value = self.get_property(name.as_str());
            if value.is_some() {
                result.push(value.unwrap())
            } else {
                break;
            }
        }
        return result;
    }

    pub fn get_property(&self, key: &str) -> Option<String> {
        let name = self.prepare_key(key);
        self.properties.get(&name).map(|s| s.clone())
    }

    fn prepare_key(&self, name: &str) -> String {
        return String::from(name).strip_prefix("${")
            .and_then(|stripped| stripped.strip_suffix("}"))
            .map_or(String::from(name), |s| s.to_string());
    }
}


/// 解析Yaml文件
impl Environment {
    pub fn read_yaml_from_file<T: AsRef<Path>>(&mut self, path: T) -> Result<(), String> {
        let conf = fs::read_to_string(path).map_err(|e| e.to_string())?;
        self.read_yaml(conf)
    }

    pub fn read_yaml(&mut self, conf: String) -> Result<(), String> {
        if let Some(loader) = self.property_loaders.get("yaml") {
            let properties: HashMap<String, String> = loader.as_ref().load(conf)?;
            if properties.len() > 0 {
                for (key, value) in properties {
                    self.properties.insert(key, value);
                }
            }
        }
        return Ok(());
    }
}

/// 属性加载器
pub trait PropertyLoader {
    /// 加载配置信息
    fn load(&self, conf: String) -> Result<HashMap<String, String>, String>;
    /// 获取此属性加载器支持的配置类型
    fn get_file_extensions(&self) -> Vec<String>;
}

