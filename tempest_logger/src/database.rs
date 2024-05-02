extern crate serde_derive;

use questdb::{
    ingress::{Buffer, Sender, TimestampMicros, TimestampNanos},
    Result,
};
use serde_json::Value;

pub struct Appender {
    db_appender: Sender,
}

impl Appender {
    pub fn new(db_url: &String) -> Appender {
        let db_appender = Sender::from_conf(format!("tcp::addr={db_url};"));
        Appender {
            db_appender: db_appender.expect("Error: failed to connecto to questdb"),
        }
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


    pub fn event_lightning(&mut self, json_object: &Value) -> Result<()> {
        println!("NOT IMPLEMENT => event_lightning: {}", json_object);
        Ok(())
    }

    pub fn event_precipitation(&mut self, json_object: &Value) -> Result<()> {
        println!("NOT IMPLEMENT => event_precipitation: {}", json_object);
        Ok(())
    }

    pub fn observation_air(&mut self, json_object: &Value) -> Result<()> {
        println!("NOT IMPLEMENT => observation_air: {}", json_object);
        Ok(())
    }

    pub fn observation_sky(&mut self, json_object: &Value) -> Result<()> {
        println!("NOT IMPLEMENT => observation_sky: {}", json_object);
        Ok(())
    }

    pub fn to_fahrenheit(&mut self, celcius: f64) -> Option<f64> {
        Some((celcius * 1.8) + 32.0)
    }

    pub fn observation_station(&mut self, json_object: &Value) -> Result<()> {
        let device_id = &json_object["device_id"]
            .as_i64()
            .expect("Error missing device id");
        let data = &json_object["obs"][0];
        let time_ms = data[0].as_i64().unwrap() * 1000000;
        let humidity = data[8].as_f64().unwrap();
        let celcius = data[7].as_f64().unwrap();
        let fahrenheit = self
            .to_fahrenheit(celcius)
            .unwrap();

        // conver metrics to english
        let wind_lull = data[1].as_f64().unwrap() * 2.2369;
        let wind_avg = data[2].as_f64().unwrap() * 2.2369; // convert to miles/hour
        let wind_gust = data[3].as_f64().unwrap() * 2.2369; // convert to miles/hour
        let rain_accum = data[12].as_f64().unwrap() * 0.0393701;// to inches
        let rain_daily = data[18].as_f64().unwrap() * 0.0393701; // to inches
        let light_dist = data[14].as_f64().unwrap() * 0.621371;
        //
        println!("observation_station: {}", data);
        let mut buffer = Buffer::new();
        buffer
            .table("tempest_station")?
            .symbol("device_id", device_id.to_string())?
            .column_f64("wind_lull", wind_lull)?
            .column_f64("wind_avg", wind_avg)?
            .column_f64("wind_gust", wind_gust)?
            .column_f64("wind_dir", data[4].as_f64().unwrap())?
            .column_f64("wind_interval", data[5].as_f64().unwrap())?
            .column_f64("pressure", data[6].as_f64().unwrap())?
            .column_f64("temperature", fahrenheit)?
            .column_f64("humidity", humidity)?
            .column_f64("luminance", data[9].as_f64().unwrap())?
            .column_f64("uv", data[10].as_f64().unwrap())?
            .column_f64("radiation", data[11].as_f64().unwrap())?
            .column_f64("rain_accum", rain_accum)?
            .column_f64("precip_type", data[13].as_f64().unwrap())?
            .column_f64("light_dist", light_dist)?
            .column_f64("light_count", data[15].as_f64().unwrap())?
            .column_f64("battery", data[16].as_f64().unwrap())?
            .column_f64("report_int", data[17].as_f64().unwrap())?
            .column_f64("rain_daily", rain_daily)?            
            .column_f64("vpd", self.calc_vpd(celcius, humidity).unwrap())?
            //
            // intentionally omitted array items 19,20,21
            //
            .column_ts("time", TimestampMicros::new(time_ms))?
            .at(TimestampNanos::now())?;

        self.db_appender.flush(&mut buffer)?;

        Ok(())
    }
}
