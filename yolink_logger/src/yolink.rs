//!
//!
//!
//!

use crate::database::Appender;
use reqwest::Error;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use std::time::UNIX_EPOCH;

pub struct Conf {
    value: Value,
}

impl Conf {
    pub fn new(config_file: &str) -> Self {
        let content = std::fs::read_to_string(config_file).unwrap();
        let value = serde_yaml::from_str::<Value>(&content).unwrap();
        Self { value: value }
    }
    pub fn get_token_url(&mut self) -> String {
        self.value["url"]["token"]
            .as_str()
            .expect("missing token url")
            .to_string()
    }
    pub fn get_api_url(&mut self) -> String {
        self.value["url"]["api"]
            .as_str()
            .expect("missing pi url")
            .to_string()
    }
    pub fn get_questdb_url(&mut self) -> String {
        self.value["url"]["questdb"]
            .as_str()
            .expect("missing questdb url")
            .to_string()
    }
    pub fn get_mqtt_broker(&mut self) -> String {
        self.value["mqtt"]["broker"]
            .as_str()
            .expect("missing mqtt broker")
            .to_string()
    }
    pub fn get_mqtt_port(&mut self) -> i64 {
        self.value["mqtt"]["port"]
            .as_i64()
            .expect("missing mqtt port")
    }
    pub fn get_ua_id(&mut self) -> String {
        self.value["security"]["ua_id"]
            .as_str()
            .expect("missing security ua_id")
            .to_string()
    }
    pub fn get_sec_id(&mut self) -> String {
        self.value["security"]["sec_id"]
            .as_str()
            .expect("missing security sec_id")
            .to_string()
    }
}

pub struct Access {}

impl Access {
    pub async fn token(url: &String, ua_id: &String, sec_id: &String) -> Result<String, Error> {
        //print!("\nconnecting to https://api.yosmart.com/open/yolink/token");
        let client = reqwest::Client::new();

        let request_body = format!("grant_type=client_credentials");
        let request = client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(ua_id, Some(sec_id))
            .body(request_body.to_string());

        let response = request.send().await?;
        let response_code = response.status().as_u16();
        match response_code {
            200 => 200,
            _ => {
                panic!("Error: HTTPs response code: {}", response_code);
            }
        };
        let json_response: String = response.text().await?;
        let value = serde_json::from_str::<Value>(&json_response).unwrap();
        //print!("\njson: {:?}", value);

        let access_token = value["access_token"]
            .as_str()
            .expect("missing access_token")
            .to_string();

        print!("\naccess_token: {:?}", access_token);
        Ok(access_token)
    }
}

#[derive(Clone, Debug)]
pub struct Device {
    id: String,
    udid: String,
    model: String,
    name: String,
    token: String,
    dtype: String,
}

pub struct Api {
    url: String,
    access_token: String,
}

impl Api {
    pub fn new(api_url: &String, access_token: &String) -> Self {
        Self {
            url: api_url.clone(),
            access_token: access_token.clone(),
        }
    }
    pub async fn get_all_devices(&mut self) -> Result<HashMap<String, Device>, Error> {
        let epoch_ms = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("epoch error")
            .as_millis();

        let request_body = json!({
            "method": "Home.getDeviceList",
            "time": epoch_ms,
        });
        let client = reqwest::Client::new();

        let request = client
            .post(self.url.to_string())
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer ".to_owned() + &self.access_token)
            .body(serde_json::to_string(&request_body).unwrap());

        let response = request.send().await?;
        let response_code = response.status().as_u16();
        match response_code {
            200 => 200,
            _ => {
                panic!("Error: failed to get the device list: {}", response_code);
            }
        };
        let json_response: String = response.text().await?;
        let json_object = serde_json::from_str::<Value>(&json_response).unwrap();

        let mut device_list = HashMap::new();
        if let serde_json::Value::Array(devices) = &json_object["data"]["devices"] {
            for d in devices {
                let device = Device {
                    id: d["deviceId"].to_string(),
                    udid: d["deviceUDID"].to_string(),
                    model: d["modelName"].to_string(),
                    name: d["name"].to_string(),
                    token: d["token"].to_string(),
                    dtype: d["type"].to_string(),
                };
                device_list.insert(device.id.clone(), device);
            }
        }

        Ok(device_list)
    }

    pub async fn get_home_id(&mut self) -> Result<String, Error> {
        let epoch_ms = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("epoch error")
            .as_millis();

        let request_body = json!({
            "method": "Home.getGeneralInfo",
            "time": epoch_ms,
        });
        let client = reqwest::Client::new();

        let request = client
            .post(self.url.to_string())
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer ".to_owned() + &self.access_token)
            .body(serde_json::to_string(&request_body).unwrap());

        let response = request.send().await?;
        let response_code = response.status().as_u16();
        match response_code {
            200 => 200,
            _ => {
                panic!("Error: failed to get the home id: {}", response_code);
            }
        };
        let json_response: String = response.text().await?;
        let json_object = serde_json::from_str::<Value>(&json_response).unwrap();

        if json_object["code"].as_str().expect("Error getting home id") != "000000" {
            panic!("Error: return code for home id: {}", json_object["code"]);
        }

        Ok(json_object["data"]["id"]
            .as_str()
            .expect("response missing home id")
            .to_string())
    }
}

pub struct MqttDatabaseLogger {
    broker: String,
    port: i64,
    topic: String,
    username: String,
}

impl MqttDatabaseLogger {
    pub fn new(
        mqtt_broker: &String,
        mqtt_port: i64,
        home_id: &String,
        access_token: &String,
    ) -> Self {
        Self {
            broker: mqtt_broker.clone(),
            port: mqtt_port,
            topic: format!("yl-home/{}/+/report", home_id),
            username: access_token.clone(),
        }
    }

    fn log_event(&mut self, db_appender: &mut Appender, message: &String) -> Result<(), Error> {
        let json_object = serde_json::from_str::<Value>(&message).unwrap();
        match json_object["event"].as_str() {
            Some("THSensor.Report") => db_appender
                .sensor_report(&json_object)
                .expect("report error"),
            Some("THSensor.Alert") => db_appender
                .sensor_alert(&json_object)
                .expect("alert error"),
            _ => println!(
                "Unknown event\n{}",
                serde_json::to_string_pretty(&json_object).unwrap()
            ),
        }
        Ok(())
    }

    pub async fn connect_to_broker(&mut self, db_appender: &mut Appender) {
        println!("\nconnecting to broker: {}:{}", self.broker, self.port);

        let mut mqttoptions = MqttOptions::new(
            "yolink.rust",
            self.broker.clone(),
            self.port.try_into().unwrap(),
        );
        mqttoptions.set_keep_alive(Duration::from_secs(20));
        mqttoptions.set_credentials(self.username.clone(), "".to_string());

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        client
            .subscribe(self.topic.clone(), QoS::AtMostOnce)
            .await
            .unwrap();
        println!("done");

        loop {
            let notification = eventloop.poll().await;
            match notification.unwrap() {
                Event::Incoming(Packet::Publish(packet)) => {
                    let message = String::from_utf8_lossy(&packet.payload).to_string();
                    let _ = self.log_event(db_appender, &message).expect("Error processing log event");
                }
                Event::Outgoing(_) => {
                   //println!(".");
                }
                _ => {
                    //println!(".");
                }
            }
        }
    }
}
