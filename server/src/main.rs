#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;
use std::env;

use diesel::r2d2::ConnectionManager;
use diesel::{r2d2};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use uuid::Uuid;
use futures::{Future};
use glob::glob;

use actix_web::{HttpServer, middleware, App, web, HttpResponse, Error};
use actix_files as af;

mod schema;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

use schema::products;

#[derive(Debug,Queryable,Serialize,Insertable,Identifiable, AsChangeset)]
#[table_name = "products"]
struct Product {
    id: Uuid,
    name: String,
    image: String,
    price: i32,
}

fn list_handler(pool: web::Data<Pool>) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move|| list(pool)).then(|res| match res {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(_) => Ok(HttpResponse::InternalServerError().into()),
    })
}

fn list(pool: web::Data<Pool>)
    -> Result<Vec<Product>, diesel::result::Error> {

    use self::schema::products::dsl::*;

    match pool.get() {
        Ok(connection) => {
            let results = products
                .load::<Product>(&connection).unwrap();

            Ok(results)
        }
        Err(_) => Err(diesel::result::Error::NotFound)
    }
}

#[derive(Debug,Deserialize)]
struct NewProduct {
    name: String,
    image: String,
    price: i32,
}

fn create_handler(new_product: web::Json<NewProduct>, pool: web::Data<Pool>)
    -> impl Future<Item = HttpResponse, Error = Error> {

    web::block(move|| create_product(new_product.into_inner(), pool)).then(|res| match res {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(_) => Ok(HttpResponse::InternalServerError().into()),
    })
}

fn create_product(new_product: NewProduct, pool: web::Data<Pool>)
    -> Result<Product, diesel::result::Error> {

    use self::schema::products::dsl::*;

    let connection = pool.get().unwrap();

    let product = Product {
        id: Uuid::new_v4(),
        name: new_product.name,
        image: new_product.image,
        price: new_product.price,
    };

    Ok(diesel::insert_into(products)
        .values(&product)
        .get_result(&connection).unwrap())
}


fn delete_handler(id: web::Path<Uuid>, pool: web::Data<Pool>)
                -> impl Future<Item = HttpResponse, Error = Error> {

    web::block(move|| delete_product(id.into_inner(), pool)).then(|res| match res {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(_) => Ok(HttpResponse::InternalServerError().into()),
    })
}

fn delete_product(product_id: Uuid, pool: web::Data<Pool>)
          -> Result<Uuid, diesel::result::Error> {

    use self::schema::products::dsl::*;

    let connection = pool.get().unwrap();

    diesel::delete(products.filter(id.eq(product_id))).execute(&connection).unwrap();

    Ok(product_id)
}


fn get_handler(id: web::Path<Uuid>, pool: web::Data<Pool>)
                  -> impl Future<Item = HttpResponse, Error = Error> {

    web::block(move|| get_product(id.into_inner(), pool)).then(|res| match res {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(_) => Ok(HttpResponse::InternalServerError().into()),
    })
}

fn get_product(product_id: Uuid, pool: web::Data<Pool>)
          -> Result<Product, diesel::result::Error> {

    use self::schema::products::dsl::*;

    match pool.get() {
        Ok(connection) => {
            let mut results = products
                .filter(id.eq(product_id))
                .limit(1)
                .load::<Product>(&connection).unwrap();

            Ok(results.pop().unwrap())
        }
        Err(_) => Err(diesel::result::Error::NotFound)
    }
}

fn update_handler(id: web::Path<Uuid>, new_product: web::Json<NewProduct>, pool: web::Data<Pool>)
               -> impl Future<Item = HttpResponse, Error = Error> {

    web::block(move|| update_product(id.into_inner(), new_product.into_inner(), pool)).then(|res| match res {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(_) => Ok(HttpResponse::InternalServerError().into()),
    })
}

fn update_product(product_id: Uuid, updates: NewProduct, pool: web::Data<Pool>)
        -> Result<Product, diesel::result::Error> {

    let connection = pool.get().unwrap();

    let product = Product{
        id: product_id,
        name: updates.name,
        price: updates.price,
        image: updates.image,
    };

    diesel::update(&product).set(&product).execute(&connection).unwrap();

    Ok(product)
}

fn images_handler()
        -> impl Future<Item = HttpResponse, Error = Error> {

    web::block(move|| list_images()).then(|res| match res {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(_) => Ok(HttpResponse::InternalServerError().into()),
    })
}

fn list_images() -> Result<Vec<String>, std::io::Error> {
    let mut result = Vec::new();

    for e in glob("./../www/img/*").expect("Failed to read glob pattern") {
        let path = format!("{}", e.unwrap().display());
        result.push(path.replace("..\\www\\img\\",""));
    }

    Ok(result)
}


fn main() -> Result<(), std::io::Error> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let pool = establish_connection();

    HttpServer::new(move || {
        println!("starting server instance...");
        App::new()
            .data(pool.clone())
            // enable logger
            .wrap(middleware::Logger::default())
            .service(web::resource("/images").route(web::get().to_async(images_handler)))
            .service(
                web::resource("/products")
                    .route(web::get().to_async(list_handler))

                    .data(web::JsonConfig::default())
                    .route(web::post().to_async(create_handler))
            )
            .service(
                web::resource("/products/{id}")
                    .route(web::get().to_async(get_handler))
                    .route(web::delete().to_async(delete_handler))
                    .data(web::JsonConfig::default())
                    .route(web::post().to_async(update_handler))
            )
            .service(
                af::Files::new("/", "./../www/").index_file("index.html"),
            )
    })
        .bind("127.0.0.1:8888")?
        .run()
}

fn establish_connection() -> Pool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
}