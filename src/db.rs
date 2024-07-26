use surrealdb::Surreal;
use surrealdb::opt::auth::Root;
use surrealdb::engine::remote::ws::{Client, Ws};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;
use surrealdb::sql::Value;
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
    // Stores a chat message in the database
    pub async fn store_message(&self, message: &ChatMessage) {
        let query = format!(
            "CREATE message SET id = '{}', message = '{}', received_at = '{}', amount = {}, currency = '{}'",
            message.id, message.message, message.received_at, message.amount, message.currency
        );
        self.client.query(query.as_str()).await.unwrap();
    }

    // Retrieves recent chat messages from the database
    pub async fn get_recent_messages(&self, max_messages: usize) -> Vec<ChatMessage> {
        let query = format!("SELECT * FROM message ORDER BY received_at DESC LIMIT {}", max_messages);
        let response: Vec<Value> = self.client.query(query.as_str()).await.unwrap().take(0).unwrap();

        response.iter().map(|val| {
            if let Value::Object(obj) = val {
                ChatMessage {
                    id: Uuid::parse_str(obj.get("id").unwrap().to_string().trim_matches('"')).unwrap(),
                    message: obj.get("message").unwrap().to_string().trim_matches('"').to_string(),
                    received_at: NaiveDateTime::parse_from_str(&obj.get("received_at").unwrap().to_string().trim_matches('"'), "%Y-%m-%d %H:%M:%S").unwrap(),
                    amount: obj.get("amount").unwrap().to_string().parse().unwrap(),
                    currency: obj.get("currency").unwrap().to_string().trim_matches('"').to_string(),
                }
            } else {
                panic!("Unexpected value type")
            }
        }).collect()
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

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub id: Uuid,
    pub message: String,
    pub received_at: NaiveDateTime,
    pub amount: f64,
    pub currency: String,
}