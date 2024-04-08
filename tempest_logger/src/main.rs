use clap::Parser;

mod database;
mod tempest;


#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: String,
}

#[tokio::main]
async fn main()  {
    let args = Args::parse();
    let mut yaml = tempest::Conf::new(&args.config);

    let mut db_appender = database::Appender::new(&yaml.get_questdb_url());

    let mut data_logger = tempest::WebsocketDatabaseLogger::new(
        &yaml.get_websocket_url(),
        &yaml.get_access_token(),
        &yaml.get_device_id(),
    );
    data_logger.ws_connect(&mut db_appender);

 }
