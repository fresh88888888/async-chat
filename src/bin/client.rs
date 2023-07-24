
use std::sync::Arc;
use async_chat::utils::ChatResult;
use async_chat::utils::{self, receive_as_json};
use async_chat::{FromClient, FromServer};
use async_std::{net, task, io};
use async_std::prelude::*;



// 从命令行获取服务器地址后，main 有一系列要调用的异步函数，因此它将函数的其余部分包装在一个异步块中，并将该块的 future 传递给 async_std::task::block_on 以运行。
fn main() -> ChatResult<()> {
    let address = std::env::args().nth(1).expect("Usage: client ADDRESS:PORT");
    task::block_on(async{
        let socket = net::TcpStream::connect(address).await?;
        socket.set_nodelay(true)?;
        let to_server = send_commands(socket.clone());
        let from_server = handle_replies(socket);
        from_server.race(to_server).await?;

        Ok(())
    })
}

/// Given a string `input`, return `Some((token, rest))`, where `token` is the first run of non-whitespace characters in `input`, and `rest` is the rest of
/// the string. If the string contains no non-whitespace characters, return `None`.
fn get_next_token(mut input: &str) -> Option<(&str, &str)> {
    input = input.trim_start();

    if input.is_empty() {
        return None;
    }

    match input.find(char::is_whitespace) {
        Some(space) => Some((&input[0..space], &input[space..])),
        None => Some((input, "")),
    }
}

/// Parse a line (presumably read from the standard input) as a `Request`.
fn parse_command(line: &str) -> Option<FromClient> {
    let (command, rest) = get_next_token(line)?;
    if command == "post" {
        let (group, rest) = get_next_token(rest)?;
        let message = rest.trim_start().to_string();
        return Some(FromClient::Post {
            group_name: Arc::new(group.to_string()),
            message: Arc::new(message.to_string()),
        });
    } else if command == "join" {
        let (group, rest) = get_next_token(rest)?;
        if !rest.trim_start().is_empty() {
            return None;
        }
        return Some(FromClient::Join {
            group_name: Arc::new(group.to_string()),
        });
    } else {
        eprintln!("Unrecognized Command: {:?}", line);
        return None;
    }
}

// 我们聊天客户端的首要职责是读取用户的命令，并将相应的数据包发送到服务器。我们将做最简单可行的事情：直接从标准输入读取行

async fn send_commands(mut to_server: net::TcpStream) -> ChatResult<()> {
    println!("Commands:\n join GROUP\n post GROUP MESSAGE...\n Type Control-D (on Unix) or Control-Z (on Windows) to close the connection.");
    let mut command_lines = io::BufReader::new(io::stdin()).lines();
    while let Some(command_result) = command_lines.next().await {
        let command = command_result?;
        // See the GitHub repo for the definition of `parse_command`.

        let request = match parse_command(&command) {
            Some(request) => request,
            None => continue,
        };

        utils::send_as_json(&mut to_server, &request).await?;
        to_server.flush().await?;
    }

    Ok(())
}

async fn handle_replies(from_server: net::TcpStream) -> ChatResult<()> {
    // 这个函数接受一个从服务器接收数据的套接字，在它周围包裹一个 async_std::io::BufReader，然后将它传递给 receive_as_json 以获取传入的 FromServer 值流。
    // 然后它使用 while let 循环来处理传入的回复，检查错误结果并打印每个服务器回复以供用户查看。
    let buffered = async_std::io::BufReader::new(from_server);
    let mut replay_stream = receive_as_json(buffered);

    while let Some(reply) = replay_stream.next().await {
        match reply? {
            FromServer::Message {
                group_name,
                message,
            } => {
                println!("message posted to {}: {}", group_name, message);
            },
            FromServer::Error(message) => {
                println!("error from server: {}", message)
            },
        }
    }

    Ok(())
}
