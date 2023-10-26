use me_rs::{logger, Server};

fn main() {
    // 最先初始化日志，让日志能被打印出来
    logger::default();
    let mut server = Server::new();
    server.run();
}
