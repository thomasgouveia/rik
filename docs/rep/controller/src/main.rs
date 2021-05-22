use std::thread;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

fn main() {
    let internal_thread = thread::spawn(move || {
        internal().unwrap();
    });
    let external_thread = thread::spawn(move || {
        external().unwrap();
    });

    internal_thread.join().unwrap();
    external_thread.join().unwrap();
}

#[actix_web::main]
async fn external() -> std::io::Result<()>{
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind("127.0.0.1:4000")?
    .run()
    .await
}

#[actix_web::main]
async fn internal() -> std::io::Result<()>{
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}