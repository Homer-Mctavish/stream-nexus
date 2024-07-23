mod exchange;
mod message;
mod sneed_env; // naming it "env" can be confusing.
mod web;

use crate::web::ChatServer;

use actix::Actor;
use anyhow::Result;
use actix_web::{
    App,
    HttpServer, 
    web::Data
};

use repository::surrealdb_repo::SurrealDBRepo;
use api::todo_api::{create_todo, get_todos, get_todo, update_todo, delete_todo}; 

mod api;
mod model;
mod repository;
mod utils;
mod prelude;
mod error;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    sneed_env::get_env();
    env_logger::init();

    let surreal = SurrealDBRepo::init().await.expect("Error connecting to SurrealDB!"); 

    let db_data = Data::new(surreal); 

    let chat = ChatServer::new(
        exchange::fetch_exchange_rates()
            .await
            .expect("Failed to fetch exchange rates."),
    )
    .start();
    let chat_for_server = chat.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone()) 
            .app_data(chat_for_server.clone())
            .service(web::javascript)
            .service(web::dashboard_javascript)
            .service(web::stylesheet)
            .service(web::dashboard_stylesheet)
            .service(web::colors)
            .service(web::chat)
            .service(web::dashboard)
            .service(web::overlay)
            .service(web::websocket)
            .service(web::logo)
            .service(create_todo) 
            .service(get_todos) 
            .service(get_todo) 
            .service(update_todo) 
            .service(delete_todo) 
    })
    //.workers(1)
    .bind(format!(
        "{}:{}",
        dotenvy::var("SERVER_IP").expect("SERVER_IP not defined."),
        dotenvy::var("SERVER_PORT").expect("SERVER_PORT not defined.")
    ))
    .expect("Could not bind requested address.")
    .run()
    .await
}




