use std::collections::HashMap;

use crate::conf::Environment;
use crate::kernel::{OrderBook, OrderBookFactory};
use crate::kernel::direct::DirectOrderBookFactory;

pub mod logger;
pub mod conf;
pub mod utils;
pub mod kernel;

/// ME撮合引擎
pub struct Server {
    /// 系统环境
    environment: Environment,
    /// OrderBook实例工厂
    order_book_factory: Box<dyn OrderBookFactory>,
    /// OrderBook列表
    order_books: HashMap<String, Box<dyn OrderBook>>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            order_books: HashMap::new(),
            environment: Environment::new(),
            order_book_factory: Box::new(DirectOrderBookFactory::new()),
        }
    }

    pub fn run(&mut self) {
        // 加载配置文件
        self.load_yaml_properties();
    }
}

impl Server {
    pub fn create_market(&mut self, code: String) {
        if self.order_books.contains_key(&code) {
            log::warn!("[Server][OrderBook]尝试创建重复的订单薄,请求被忽略.Code={code}");
            return;
        }
        let mut order_book = self.order_book_factory.create();
        self.order_books.insert(code.clone(), order_book);
    }
}

impl Server {
    fn load_yaml_properties(&mut self) {
        self.load_yaml_file("resources/application.yaml");
        let profiles = self.environment.get_properties("application.profiles");
        if profiles.len() > 0 {
            for (_, profile) in profiles.iter().enumerate() {
                let file_name = format!("resources/application-{profile}.yaml");
                self.load_yaml_file(&file_name);
            }
        }
    }

    fn load_yaml_file(&mut self, file_name: &str) {
        if let Ok(()) = self.environment.read_yaml_from_file(&file_name) {
            log::info!("[Env]已加载配置{file_name}")
        };
    }
}