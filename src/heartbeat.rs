use std::collections::HashMap;

use crate::socket::SocketMap;

pub struct HeartbeatSocketData {
    pub time_left: u32,
}

pub struct Heartbeat {
    pub all_sockets: SocketMap,
    pub current_sockets: HashMap<String, HeartbeatSocketData>,
}

impl Heartbeat {
    pub fn new(socket_map: SocketMap) -> Self {
        Self {
            all_sockets: socket_map,
            current_sockets: HashMap::new(),
        }
    }

    pub async fn beat(self: &mut Self) {
        let mut keys: Vec<String> = Vec::new();
        {
            for key in self.all_sockets.read().await.keys() {
                keys.push(key.clone());
            }
        }

        self.remove_removed(&keys);
        self.add_added(&keys);

        for (token, data) in &mut self.current_sockets {
            if data.time_left == 0 {
                data.time_left = 42;
            }

            data.time_left = data.time_left - 1;

            if data.time_left == 0 {
                {
                    self.all_sockets.write().await.get_mut(token).unwrap().send_heartbeat().await;
                }
            }
        }
    }

    pub fn remove_removed(self: &mut Self, online_sockets: &Vec<String>) {
        let mut to_delete: Vec<String> = Vec::new();
        for socket in self.current_sockets.keys() {
            if !online_sockets.contains(socket) {
                to_delete.push(socket.clone());
            }
        }

        for socket in to_delete {
            self.current_sockets.remove(&socket);
        }
    }

    pub fn add_added(self: &mut Self, online_sockets: &Vec<String>) {
        let mut to_add: Vec<String> = Vec::new();
        for socket in online_sockets {
            if !self.current_sockets.contains_key(socket) {
                to_add.push(socket.clone());
            }
        }

        for socket in to_add {
            self.current_sockets
                .insert(socket, HeartbeatSocketData { time_left: 0 });
        }
    }
}
