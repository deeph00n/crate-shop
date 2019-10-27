use actix_web::{web, HttpResponse};
use orientdb_client::{OSession};
use serde::{Serialize};


#[derive(Debug, Serialize)]
struct Product {
    name: String,
    image: String,
    price: i32,
}

pub fn list(session: web::Data<OSession>) -> HttpResponse {
    let results = session
        .query("select Name,Price,Image.Name from Product")
        .run()
        .expect("Unable to query the class Product")
        .map(|e| {
            e.map(|i| Product {
                name: i.get("Name"),
                price: i.get("Price"),
                image: i.get("Image.Name"),
            })
        })
        .collect::<Result<Vec<Product>, _>>().expect("Unable to collect result");

    HttpResponse::Ok().json(results)
}