use actix_web::{web, HttpResponse};
use orientdb_client::{OSession};

pub fn list(session: web::Data<OSession>) -> HttpResponse {
    let query = format!("select Data from Image where Name = '{}'", "Selfie.jpg");
    let mut data = 0;
    let results = session
        .query(query)
        .run()
        .expect("Unable to query the class Image")
        .map(|e| {
            e.map(|i|
                data = i.get("Data")
            )
        })
        .collect::<Result<Vec<Product>, _>>().expect("Unable to collect result");

    HttpResponse::Ok().json(results)
}
