use std::sync::Arc;

use serde::{Serialize, Deserialize};

pub mod utils;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum FromClient {
    Join{
        group_name: Arc<String>,
    },
    Post{
        group_name: Arc<String>,
        message: Arc<String>,
    },
}


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum FromServer {
    Message {
        group_name: Arc<String>,
        message: Arc<String>,
    },
    Error(String),
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fromclient_json() {
        // FromClient 代表客户端可以发送到服务器的数据包：它可以请求加入一个房间并将消息发布到它已加入的任何房间。FromServer 表示服务器可以发回的内容：发布到某个组的消息和错误消息。
        // 使用引用计数的 Arc<String> 而不是普通的 String 有助于服务器在管理组和分发消息时避免复制字符串。

        let from_client = FromClient::Post { group_name: Arc::new("Dogs".to_string()), message: Arc::new("Samoyeds rock!".to_string()) };
        let json = serde_json::to_string(&from_client).unwrap();
        
        // #[derive] 属性告诉 serde 为 FromClient 和 FromServer 生成其 Serialize 和 Deserialize 的实现。 这让我们可以调用 serde_json::to_string 将它们转换为 JSON 值，
        // 通过网络发送它们，最后调用 serde_json::from_str 将它们转换回 Rust 形式。
        assert_eq!(json, r#"{"Post":{"group_name":"Dogs","message":"Samoyeds rock!"}}"#);
        assert_eq!(serde_json::from_str::<FromClient>(&json).unwrap(), from_client);
    }
}
