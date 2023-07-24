use std::error::Error;

use async_std::prelude::*;
use serde::Serialize;
use std::marker::Unpin;

// 这些是通用错误类型。async_std、serde_json 和 tokio 都定义了自己的错误类型，但是它们都实现了标准库的 From，? 运算符可以自动将它们全部转换为 ChatError，
// 可以将任何合适的错误类型转换为 Box<dyn Error + Send + Sync + 'static>。

// Send 和 Sync 边界确保如果分发到另一个线程的任务失败，它可以安全地将错误报告给主线程。
pub type ChatError = Box<dyn Error + Send + Sync + 'static>;
pub type ChatResult<T> = Result<T, ChatError>;

pub async fn send_as_json<S, P>(outbound: &mut S, packet: &P) -> ChatResult<()>
where
    S: async_std::io::Write + Unpin,
    P: Serialize,
{
    let mut json = serde_json::to_string(&packet)?;
    json.push('\n');
    outbound.write_all(json.as_bytes()).await?;

    Ok(())
}

use serde::de::DeserializeOwned;

pub fn receive_as_json<S, P>(inbound: S) -> impl Stream<Item = ChatResult<P>>
where
    S: async_std::io::BufRead + Unpin,
    P: DeserializeOwned,
{
    inbound.lines().map(|line_result| -> ChatResult<P>{
        let line = line_result?;
        let parsed = serde_json::from_str::<P>(&line)?;
        Ok(parsed)
    })
}
