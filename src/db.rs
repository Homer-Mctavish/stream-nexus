use surrealdb::Surreal;
use surrealdb::opt::auth::Root;
use surrealdb::engine::remote::ws::{Client, Ws};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Database {
    client: Surreal<Client>,
}

impl Database {
    // Initializes a new database connection
    pub async fn new() -> Self {
        let client = Surreal::new::<Ws>("localhost:8000").await.unwrap();

        // Sign in as root user
        client.signin(Root {
            username: "root",
            password: "root",
        }).await.unwrap();

        // Select namespace and database
        client.use_ns("test").use_db("test").await.unwrap();

        Database { client }
    }

    // Retrieves all users from the database
    pub async fn get_all_users(&self) -> Vec<User> {
        let mut response = self.client.query("SELECT * FROM user").await.unwrap();
        response.take::<Vec<User>>(0).unwrap()
    }

    // Creates a new user in the database
    pub async fn create_user(&self, user: NewUser) -> User {
        let mut response = self.client.query(format!("CREATE user SET name = '{}'", user.name)).await.unwrap();
        response.take::<Option<User>>(0).unwrap().unwrap()
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
