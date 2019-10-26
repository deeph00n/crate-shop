use actix_files as fs;
use actix_web::{web, App, HttpServer, middleware};
use orientdb_client::{OrientDB};
use ron::de::from_reader;
use serde::{Deserialize};
use std::{fs::File};

mod products;

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        let config = read_config();
        let client = OrientDB::connect((config.db_host,config.db_port))
            .expect("Unable to get orientDb client");
        let session = client.session(&config.db_database, &config.db_user,&config.db_password)
            .expect("Unable to create session");

        App::new()
            .data(session)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .service(
                web::resource("/product").route(web::get().to(products::list))
            )
            .service(
                fs::Files::new("/", "./../www/").index_file("index.html"),
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
}

#[derive(Debug, Deserialize)]
struct Config {
    db_host: String,
    db_port: u16,
    db_database: String,
    db_user: String,
    db_password: String,
}

fn read_config() -> Config {
    let f = File::open("config.ron").expect("Failed opening config.ron");
    let config: Config = from_reader(f).expect("Could no parse config file");
    config
}