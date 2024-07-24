use surrealdb::{Surreal, Response};
use surrealdb::opt::auth::Root;
use surrealdb::engine::remote::http::Http;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Clone)]
pub struct Database {
    client: Surreal<Http>,
}

impl Database {
    pub async fn new() -> Self {
        let client = Surreal::new::<Http>("http://localhost:8000").await.unwrap();

        // Sign in as root user
        client.signin(Root {
            username: "root",
            password: "root",
        }).await.unwrap();

        // Select namespace and database
        client.use_ns("test").use_db("test").await.unwrap();

        Database { client }
    }

    pub async fn get_all_users(&self) -> Vec<User> {
        let mut response = self.client.query("SELECT * FROM user").await.unwrap();
        response.take::<Vec<User>>(0).unwrap()
    }

    pub async fn create_user(&self, user: NewUser) -> User {
        let mut response = self.client.query(format!("CREATE user SET name = '{}'", user.name)).await.unwrap();
        response.take::<User>(0).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewUser {
    pub name: String,
}
