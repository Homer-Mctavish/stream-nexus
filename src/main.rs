mod exchange;
mod message;
mod sneed_env; // naming it "env" can be confusing.
mod web;
mod db;

use crate::web::ChatServer;
use actix::Actor;
use actix_web::{web as other_web, App, HttpServer, Responder, HttpResponse};
use anyhow::Result;
use std::sync::Mutex;
use db::{Database, NewUser};
use serde::Deserialize;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::sql::Id;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;

#[derive(Debug, Deserialize)]
struct CustomThing {
    id: Id,
}

#[derive(Debug, Deserialize)]
struct Person {
    id: CustomThing,
    name: String,
    age: u8,
}

struct SurrealDBData {
    db: Surreal<Client>
}

impl SurrealDBData {
    async fn new() -> surrealdb::Result<Self> {
        let db = Surreal::new::<Ws>("localhost:8000").await?;
        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await?;
        db.use_ns("namespace").use_db("database").await?;
        Ok(Self { db })
    }

    async fn create_person(&self) -> surrealdb::Result<()> {
        self.db.query("CREATE person:john SET name = 'John Doe', age = 25").await?.check()?;
        Ok(())
    }

    async fn get_person(&self) -> surrealdb::Result<Option<Person>> {
        let john: Option<Person> = self.db.select(("person", "john")).await?;
        Ok(john)
    }
}

async fn get_users(db: other_web::Data<Mutex<Database>>) -> impl Responder {
    let db = db.lock().unwrap();
    let users = db.get_all_users().await;
    HttpResponse::Ok().json(users)
}

async fn create_user(db: other_web::Data<Mutex<Database>>, user: other_web::Json<NewUser>) -> impl Responder {
    let db = db.lock().unwrap();
    let user = db.create_user(user.into_inner()).await;
    HttpResponse::Ok().json(user)
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    sneed_env::get_env();
    env_logger::init();

    let db = other_web::Data::new(Mutex::new(Database::new().await));

    let chat = ChatServer::new(
        exchange::fetch_exchange_rates()
            .await
            .expect("Failed to fetch exchange rates."),
    )
    .start();
    let chat_for_server = chat.clone();

    // Initialize SurrealDB connection
    let surreal_db = SurrealDBData::new().await.expect("Failed to connect to SurrealDB");

    // Create a person in SurrealDB (example)
    surreal_db.create_person().await.expect("Failed to create person");

    // Query that person (example)
    let john = surreal_db.get_person().await.expect("Failed to get person");
    dbg!(john);

    HttpServer::new(move || {
        App::new()
            .app_data(chat_for_server.clone())
            .app_data(db.clone())
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
            .route("/", other_web::get().to(get_users))
            .route("/user", other_web::post().to(create_user))
    })
    .bind(format!(
        "{}:{}",
        dotenvy::var("SERVER_IP").expect("SERVER_IP not defined."),
        dotenvy::var("SERVER_PORT").expect("SERVER_PORT not defined.")
    ))
    .expect("Could not bind requested address.")
    .run()
    .await
}

