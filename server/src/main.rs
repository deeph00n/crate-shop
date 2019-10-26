use actix_files as fs;
use actix_web::{web, App, HttpServer, middleware, HttpResponse};
use orientdb_client::{OrientDB, OSession};
use ron::de::from_reader;
use serde::{Deserialize,Serialize};
use std::{fs::File};


#[derive(Debug, Serialize)]
struct Product {
    name: String,
    price: i32,
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

fn list_products(session: web::Data<OSession>) -> HttpResponse {
    let results = session
        .query("select from Product")
        .run()
        .expect("Unable to query the class Product")
        .map(|e| {
            e.map(|i| Product {
                name: i.get("Name"),
                price: i.get("Price"),
            })
        })
        .collect::<Result<Vec<Product>, _>>().unwrap();
    HttpResponse::Ok().json(results)
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        let config = read_config();
        let client = OrientDB::connect((config.db_host,config.db_port)).unwrap();
        let session = client.session(&config.db_database, &config.db_user,&config.db_password).unwrap();

        App::new()
            .data(session)
            // enable logger
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .service(web::resource("/product").route(web::get().to(list_products)))
            .service(
                // static files
                fs::Files::new("/", "./../www/").index_file("index.html"),
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
}