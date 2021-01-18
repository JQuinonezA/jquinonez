mod config;
use actix_web::{middleware, get, post, web, App, HttpResponse, HttpServer, Responder, Error};
use actix_web::http::{StatusCode};
use std::io;
use std::io::Write;
use dotenv::dotenv;
use tokio_postgres::NoTls;
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
//use postgres::{Client, NoTls};

async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        let filepath = format!("./tmp/{}", sanitize_filename::sanitize(&filename));

        
        let mut f = web::block(|| std::fs::File::create(filepath))
            .await
            .unwrap();

        
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }
    }
    Ok(HttpResponse::Ok().into())
}
async fn home() -> Result<HttpResponse, Error> {
    Ok(
        HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/index.html"))
    )
}

#[actix_web::main]
async fn main() -> io::Result<()>{
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    std::fs::create_dir_all("./tmp").unwrap();
    dotenv().ok();
    let config = crate::config::Config::from_env().unwrap();
    //let pool = config.pg.create_pool(NoTls).unwrap();

    println!("Starting servert at http://{}:{}/", config.server.host, config.server.port);

    HttpServer::new(|| {
        App::new().wrap(middleware::Logger::default()).service(
            web::resource("/")
            .route(web::get().to(home))
            .route(web::post().to(save_file)),
        )
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
    

}

