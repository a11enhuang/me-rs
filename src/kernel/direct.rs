//! 基于链表实现的OrderBook
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::kernel::{Order, OrderAction, OrderBook, OrderBookFactory, TimeInForce};

//---------------------------------------------------
//                 OrderBook
//---------------------------------------------------

/// 基于链表实现的OrderBook
pub struct DirectOrderBook {
    /// 订单簿卖单的第一个元素
    first_ask_order: Option<Rc<RefCell<DirectOrder>>>,
    // /// 订单簿买单的第一个元素
    first_bid_order: Option<Rc<RefCell<DirectOrder>>>,
    /// 委托单的索引
    order_index: BTreeMap<u64, Rc<DirectOrder>>,
    /// ask委托单列表
    ask_order_buckets: BTreeMap<i64, Rc<RefCell<Bucket>>>,
    /// bid委托单列表
    bid_order_buckets: BTreeMap<i64, Rc<RefCell<Bucket>>>,
}

impl DirectOrderBook {
    // fn try_match_instantly(&mut self, order: DirectOrder) -> u64 {
    //     let is_bid_action = order.order_action == OrderAction::BID;
    //     let limit_price = order.price;
    //     if is_bid_action {
    //         let first_ask_order = self.first_ask_order.clone();
    //         // 如果没有深度无法成交，直接返回0
    //         if first_ask_order.is_none() {
    //             return order.filled as u64;
    //         }
    //         // 如果没有对手盘根本没法成交，直接返回0
    //         let first_ask_order = first_ask_order.unwrap();
    //         if first_ask_order.price > limit_price {
    //             return order.filled as u64;
    //         }
    //     } else {
    //         let first_bid_order = self.first_bid_order.clone();
    //         // 如果没有深度无法成交，直接返回0
    //         if first_bid_order.is_none() {
    //             return order.filled as u64;
    //         }
    //         // 如果没有对手盘根本没法成交，直接返回0
    //         let first_bid_order = first_bid_order.unwrap();
    //         if first_bid_order.price < limit_price {
    //             return order.filled as u64;
    //         }
    //     }
    //     // 获取剩余成交数量
    //     let mut remain_size = order.size - order.filled;
    //
    //     return 0;
    // }

    fn insert_order(&mut self, order: DirectOrder, free_bucket: Option<Rc<RefCell<Bucket>>>) {
        let order = Rc::new(RefCell::new(order));
        let mut order_ref = order.borrow_mut();
        let is_ask_order = order_ref.order_action == OrderAction::ASK;
        let mut buckets = if is_ask_order {
            &mut self.ask_order_buckets
        } else {
            &mut self.bid_order_buckets
        };
        match buckets.get(&order_ref.price).clone() {
            Some(to_bucket) => {
                // 找到当前价位的桶
                let mut to_bucket_ref = to_bucket.borrow_mut();
                // 更新挂单数和挂单额
                to_bucket_ref.order_number += 1;
                to_bucket_ref.total_volume += order_ref.size - order_ref.filled;
                // 获取链表当前尾节点,链接下一个档位
                let old_tail = to_bucket_ref.get_tail().unwrap();
                let mut old_tail_ref = old_tail.borrow_mut();

                // 更新下一个节点为当前节点
                to_bucket_ref.tail = Some(order.clone());

                // 获取链表的上一个节点
                let prev_order = old_tail_ref.get_prev();
                if let Some(prev) = &prev_order {
                    prev.borrow_mut().next = Some(order.clone())
                }

                old_tail_ref.prev = Some(order.clone());

                order_ref.next = Some(old_tail.clone());
                order_ref.prev = prev_order.clone();
                order_ref.parent = Some(to_bucket.clone())
            }
            None => {
                let new_bucket = free_bucket.unwrap_or_else(|| Rc::new(RefCell::new(Bucket {
                    total_volume: 0,
                    order_number: 0,
                    tail: None,
                })));
                let mut new_bucket_ref = new_bucket.borrow_mut();
                new_bucket_ref.tail = Some(order.clone());
                new_bucket_ref.total_volume = order_ref.size - order_ref.filled;
                new_bucket_ref.order_number = 1;
                order_ref.parent = Some(new_bucket.clone());
                buckets.insert(order_ref.price, new_bucket.clone());

                let mut lb: Option<Rc<RefCell<Bucket>>> = None;

                match lb {
                    Some(lower_bucket) => {}
                    None => {
                        let old_best_order = if is_ask_order {
                            self.first_ask_order.clone()
                        } else {
                            self.first_bid_order.clone()
                        };

                        if let Some(old_best_order) = &old_best_order {
                            old_best_order.borrow_mut().next = Some(order.clone());
                        }

                        if is_ask_order {
                            self.first_ask_order = Some(order.clone());
                        } else {
                            self.first_bid_order = Some(order.clone());
                        }

                        order_ref.next = None;
                        order_ref.prev = old_best_order.clone();
                    }
                }
            }
        }
        return;
    }
}

impl OrderBook for DirectOrderBook {
    fn place_order(&mut self, order: Order) {
        let mut dorder = DirectOrder {
            order_id: 0,
            price: 10,
            size: 12,
            filled: 0,
            timestamp: 0,
            order_action: OrderAction::BID,
            time_in_force: None,
            parent: None,
            prev: None,
            next: None,
        };
        self.insert_order(dorder, None);
    }
}

//---------------------------------------------------
//                 Order
//---------------------------------------------------


/// Bucket 订单簿中每个价位的节点.
///
///      |-----bucket-----|
///      | total_volume   |
///      | order_number   |
///      |  ----tail----  |
///      |  |   order  |  |
///      |  |   order  |  |
///      |  |   order  |  |
///      |  ------|-----  |
///      ---------|--------
///               |               |-----bucket-----|
///               |               | total_volume   |
///               |               | order_number   |
///               |               |  ----tail----  |
///               |--------------------> order  |  |
///                               |  |   order  |  |
///                               |  |   order  |  |
///                               |  ------------  |
///                               ------------------
///
///
struct Bucket {
    /// 该价位的总挂单量
    total_volume: i64,
    /// 该价位的总委托单数量
    order_number: u64,
    /// 下一个档位的首单
    tail: Option<Rc<RefCell<DirectOrder>>>,
}

impl Bucket {
    fn get_tail(&self) -> Option<Rc<RefCell<DirectOrder>>> {
        return self.tail.clone();
    }
}

/// 委托单
struct DirectOrder {
    /// 委托单ID
    order_id: u64,
    /// 委托价
    price: i64,
    /// 委托数量
    size: i64,
    /// 已成交数量
    filled: i64,
    /// 委托时间
    timestamp: u64,
    /// 下单类型
    order_action: OrderAction,
    /// limit单的下单方式
    time_in_force: Option<TimeInForce>,
    /// 连接上一个链表
    parent: Option<Rc<RefCell<Bucket>>>,
    /// 上一个委托单
    prev: Option<Rc<RefCell<DirectOrder>>>,
    /// 下一个委托单
    next: Option<Rc<RefCell<DirectOrder>>>,
}


impl DirectOrder {
    fn get_parent(&self) -> Option<Rc<RefCell<Bucket>>> {
        self.parent.clone()
    }

    fn get_prev(&self) -> Option<Rc<RefCell<DirectOrder>>> {
        self.prev.clone()
    }

    fn get_next(&self) -> Option<Rc<RefCell<DirectOrder>>> {
        self.next.clone()
    }
}

//---------------------------------------------------
//                 OrderBookFactory
//---------------------------------------------------

/// 创建DirectOrderBook实例的工厂
pub struct DirectOrderBookFactory {}

impl DirectOrderBookFactory {
    /// 创建一个DirectOrderBookFactory实例
    pub fn new() -> DirectOrderBookFactory {
        DirectOrderBookFactory {}
    }
}

impl OrderBookFactory for DirectOrderBookFactory {
    fn create(&self) -> Box<dyn OrderBook> {
        Box::new(DirectOrderBook {
            ask_order_buckets: BTreeMap::new(),
            bid_order_buckets: BTreeMap::new(),
            first_ask_order: None,
            first_bid_order: None,
            order_index: BTreeMap::new(),
        })
    }
}