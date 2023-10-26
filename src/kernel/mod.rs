pub mod direct;

#[derive(PartialEq)]
pub enum TimeInForce {
    GTC,
    IOC,
    FOK,
}

#[derive(PartialEq)]
pub enum OrderAction {
    BID,
    ASK,
}

/// OrderBook
pub trait OrderBook {
    fn place_order(&mut self, order: Order);
}

pub struct Order {
    pub price: i64,
    pub order_action: OrderAction,
    pub time_in_force: Option<TimeInForce>,
}

/// 这是一个工厂模式的实现，用于创建OrderBook实例
pub trait OrderBookFactory {
    /// 创建OrderBook实例
    fn create(&self) -> Box<dyn OrderBook>;
}