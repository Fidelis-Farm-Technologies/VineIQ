use clap::Parser;
use reqwest::Error;

mod database;
mod yolink;


#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let mut yaml = yolink::Conf::new(&args.config);

    let access_token =
        yolink::Access::token(&yaml.get_token_url(), &yaml.get_ua_id(), &yaml.get_sec_id()).await?;

    let mut yolink_api = yolink::Api::new(&yaml.get_api_url(), &access_token);
    let device_list = yolink_api
        .get_all_devices()
        .await
        .expect("Error acquiring the device list");

    let home_id = yolink_api.get_home_id().await?;

    let mut db_appender = database::Appender::new(&yaml.get_questdb_url());
    let mut database_logger = yolink::MqttDatabaseLogger::new(
        &yaml.get_mqtt_broker(),
        yaml.get_mqtt_port(),
        &home_id,
        &access_token
    );
    database_logger.connect_to_broker(&mut db_appender).await;

    Ok(())
}
