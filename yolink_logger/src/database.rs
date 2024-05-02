extern crate serde_derive;

//use chrono::NaiveDateTime;
use questdb::{
    ingress::{Buffer, Sender, TimestampMicros, TimestampNanos},
    Result,
};
use serde_json::Value;

pub struct Appender {
    db_appender: Sender,
}

impl Appender {
    /*
      {
      "data": {
        "alarm": {
          "code": 0,
          "highHumidity": false,
          "highTemp": false,
          "lowBattery": false,
          "lowHumidity": false,
          "lowTemp": false,
          "period": false
        },
        "battery": 4,
        "batteryType": "Li",
        "humidity": 32.5,
        "humidityCorrection": 0,
        "humidityLimit": {
          "max": 100,
          "min": 0
        },
        "interval": 0,
        "loraInfo": {
          "gatewayId": "d88b4c1603046d08",
          "gateways": 2,
          "netId": "010201",
          "signal": -67
        },
        "mode": "f",
        "state": "normal",
        "tempCorrection": 0,
        "tempLimit": {
          "max": 35,
          "min": 1.4
        },
        "temperature": 18.4,
        "version": "050f"
      },
      "deviceId": "d88b4c010008b987",
      "event": "THSensor.Report",
      "msgid": "1712517507810",
      "time": 1712517507811
    }
    */
    pub fn new(db_url: &String) -> Appender {
        let db_appender = Sender::from_conf(format!("tcp::addr={db_url};"));
        Appender {
            db_appender: db_appender.expect("Error: failed to connecto to questdb"),
        }
    }

    pub fn to_fahrenheit(&mut self, celcius: f64) -> Option<f64> {
        Some((celcius * 1.8) + 32.0)
    }

    pub fn calc_vpd(&mut self, celcius: f64, rh: f64) -> Option<f64> {
        let base: f64 = 10.0;
        let exp: f64 = 7.5 * celcius / (237.3 + celcius);

        let vp_saturated: f64 = (610.7 * base.powf(exp)) / 1000.00;
        //let vp_actual: f64 = ((rh * vp_saturated) / 100.0) / 1000.0;
        let vpd: f64 = ((100.0 - rh) / 100.0) * vp_saturated;

        //println!("vp saturated: {}", vp_saturated/1000.0);
        //println!("vp actual: {}", vp_actual/1000.0);
        //println!("vpd : {}", vpd);

        Some(vpd)
    }

    pub fn sensor_alert(&mut self, data: &Value) -> Result<()> {
        println!(
            "sensor_alert: \n{}",
            serde_json::to_string_pretty(&data).unwrap()
        );
        Ok(())
    }
    pub fn sensor_report(&mut self, json_object: &Value) -> Result<()> {
        println!("sensor_report:");
        let time_ms = json_object["time"].as_i64().unwrap() * 1000; // microseconds
        let device_id = json_object["deviceId"].as_str().expect("Missing deviceId");
        let data = &json_object["data"];
        let celcius = data["temperature"].as_f64().unwrap();
        let fahrenheit = self.to_fahrenheit(celcius).unwrap();
        let humidity = data["humidity"].as_f64().unwrap();
        let vpd = self.calc_vpd(celcius, humidity).unwrap();

        let mut buffer = Buffer::new();
        let _ = buffer
            .table("yolink")?
            .symbol("deviceId", device_id)?
            .symbol("gatewayId", data["loraInfo"]["gatewayId"].as_str().unwrap())?
            .symbol("netId", data["loraInfo"]["netId"].as_str().unwrap())?
            .symbol("mode", data["mode"].as_str().unwrap())?
            .symbol("state", data["state"].as_str().unwrap())?
            .column_f64("temperature", fahrenheit)?
            .column_f64("humidity", humidity)?
            .column_f64("vpd", vpd)?
            .column_i64("battery", data["battery"].as_i64().unwrap())?
            .column_bool("lowBattery", data["alarm"]["lowBattery"].as_bool().unwrap())?
            .column_i64("signal", data["loraInfo"]["signal"].as_i64().unwrap())?
            .column_ts("time", TimestampMicros::new(time_ms))?
            .at(TimestampNanos::now())
            .unwrap();
        let _ = self.db_appender.flush(&mut buffer).unwrap();

        Ok(())
    }
}
