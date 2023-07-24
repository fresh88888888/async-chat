use std::sync::Arc;
use async_std::prelude::*;
use async_chat::utils::ChatResult;
use connection::serve;


mod group_tables;
mod connection;
mod group;

fn main() -> ChatResult<()>{
    let address = std::env::args().nth(1).expect("Usage: server ADDRESS");
    let chat_group_table = Arc::new(group_tables::GroupTables::new());

    async_std::task::block_on(async {
        // This code was shown in the chapter introduction.
        let listener = async_std::net::TcpListener::bind(address).await?;
        let mut new_connections = listener.incoming();

        while let Some(socket_result) = new_connections.next().await {
            let socket = socket_result?;
            let groups = chat_group_table.clone();

            async_std::task::spawn(async {
                log_error(serve(socket, groups).await);
            });
        }
        Ok(())
    })
}

fn log_error(result: ChatResult<()>) {
    if let Err(error) = result {
        eprintln!("Error: {}", error);
    }
}