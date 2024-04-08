//!
//!
//!
//!

use crate::database::Appender;
use serde_json::Value;
use url::Url;

pub struct Conf {
    value: Value,
}

impl Conf {
    pub fn new(config_file: &str) -> Self {
        let content = std::fs::read_to_string(config_file).unwrap();
        let value = serde_yaml::from_str::<Value>(&content).unwrap();
        Self { value: value }
    }
    pub fn get_access_token(&mut self) -> String {
        self.value["access_token"]
            .as_str()
            .expect("missing access token")
            .to_string()
    }
    pub fn get_device_id(&mut self) -> String {
        self.value["device_id"]
            .as_str()
            .expect("missing station id")
            .to_string()
    }
    pub fn get_websocket_url(&mut self) -> String {
        self.value["websocket_url"]
            .as_str()
            .expect("missing api url")
            .to_string()
    }
    pub fn get_questdb_url(&mut self) -> String {
        self.value["questdb"]
            .as_str()
            .expect("missing questdb url")
            .to_string()
    }
}

pub struct WebsocketDatabaseLogger {
    websocket_url: String,
    access_token: String,
    device_id: String,
}

impl WebsocketDatabaseLogger {
    pub fn new(websocket_url: &String, access_token: &String, device_id: &String) -> Self {
        Self {
            websocket_url: websocket_url.clone(),
            access_token: access_token.clone(),
            device_id: device_id.clone(),
        }
    }

    pub fn ws_connect(&mut self, db_appender: &mut Appender) {
        let ws_url =
            Url::parse(format!("{}?token={}", self.websocket_url, self.access_token).as_str())
                .unwrap();

        println!("ws_url: {}", ws_url);
        let (mut socket, response) = tungstenite::connect(ws_url).expect("Error connecting");
        println!("Response HTTP code: {}", response.status());

        let listen_command = format!(
            "{{\"type\":\"listen_start\",\"device_id\": {},\"id\":\"vineiq-{}\"}}",
            self.device_id, self.device_id
        );
        socket.send(listen_command.into()).expect("Error sending listen command");

        loop {
            let msg: tungstenite::Message = socket.read().expect("Error reading message");
            let msg = match msg {
                tungstenite::Message::Text(s) => s,
                _ => {
                    panic!()
                }
            };
            let parsed: Value = serde_json::from_str(&msg).expect("Error parsing JSON");
            let record_type = parsed["type"].to_string();
            match record_type.as_str() {
                "\"obs_air\"" => db_appender
                    .observation_air(&parsed)
                    .expect("Failed to insert record"),
                "\"obs_sky\"" => db_appender
                    .observation_sky(&parsed)
                    .expect("Failed to insert record"),
                "\"obs_st\"" => db_appender
                    .observation_station(&parsed)
                    .expect("Failed to insert record"),
                "\"evt_strike\"" => db_appender
                    .event_lightning(&parsed)
                    .expect("Failed to insert record"),
                "\"evt_precip\"" => db_appender
                    .event_precipitation(&parsed)
                    .expect("Failed to insert record"),
                "\"ack\"" => println!("ack: {}", parsed),
                _ => println!("status: {}\n", parsed),
            }
        }
    }
}
