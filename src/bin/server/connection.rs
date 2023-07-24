

use std::sync::Arc;

use async_chat::{utils::{ChatResult, self}, FromServer, FromClient};
use async_std::{net::TcpStream, sync::Mutex, io::{WriteExt, self}, stream::StreamExt};

use crate::group_tables::GroupTables;

pub struct OutBound(Mutex<TcpStream>);

impl OutBound {
    pub fn new(to_client: TcpStream) -> OutBound {
        OutBound(Mutex::new(to_client))
    }

    pub async fn send(&self, packet: FromServer) -> ChatResult<()> {
        let mut guard = self.0.lock().await;
        utils::send_as_json(&mut *guard, &packet).await?;
        guard.flush().await?;

        Ok(())
    }
}


pub async fn serve(socket: TcpStream, groups: Arc<GroupTables>) -> ChatResult<()> {
    let outbound = Arc::new(OutBound::new(socket.clone()));
    let buffered = io::BufReader::new(socket);
    let mut from_client = utils::receive_as_json(buffered);

    while let Some(request_result) = from_client.next().await {
        let request = request_result?;

        let result = match request {
            FromClient::Join { group_name } => {
                let group = groups.get_or_create(group_name);
                group.join(outbound.clone());

                Ok(())
            },
            FromClient::Post { group_name, message } => {
                match groups.get(&group_name) {
                    Some(group) => {
                        group.post(message);

                        Ok(())
                    },
                    None => {
                        Err(format!("Group `{}` doesn't exist", group_name))
                    }
                }
            }
        };

        if let Err(message) = result {
            let report = FromServer::Error(message);
            outbound.send(report).await?;
        }
    }

    Ok(())
}