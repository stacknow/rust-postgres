use actix_web::{web, App, HttpServer, HttpResponse};
use tokio_postgres::{NoTls};
use deadpool_postgres::{Config, Pool};
use std::env;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    email: String,
}

// Initialize and configure the database pool
async fn init_db_pool() -> Pool {
    dotenv::dotenv().ok();
    let mut cfg = Config::new();
    cfg.dbname = Some("rust_postgres_db".into());
    cfg.host = Some("localhost".into());
    cfg.user = Some(env::var("DATABASE_USER").expect("DATABASE_USER not set"));
    cfg.password = Some(env::var("DATABASE_PASSWORD").expect("DATABASE_PASSWORD not set"));

    cfg.create_pool(None, NoTls).expect("Failed to create pool")
}

// Route to get all users
async fn get_users(db_pool: web::Data<Pool>) -> HttpResponse {
    let client = db_pool.get().await.expect("Error connecting to the database");
    let statement = client.prepare("SELECT id, name, email FROM users").await.unwrap();
    let users = client.query(&statement, &[])
        .await
        .unwrap()
        .iter()
        .map(|row| User {
            id: row.get(0),
            name: row.get(1),
            email: row.get(2),
        })
        .collect::<Vec<User>>();

    HttpResponse::Ok().json(users)
}

// Route to create a new user
async fn create_user(db_pool: web::Data<Pool>, user: web::Json<User>) -> HttpResponse {
    let client = db_pool.get().await.expect("Error connecting to the database");
    let statement = client
        .prepare("INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id")
        .await
        .unwrap();
    let rows = client
        .query(&statement, &[&user.name, &user.email])
        .await
        .unwrap();

    let user_id: i32 = rows[0].get(0);
    HttpResponse::Created().json(User {
        id: user_id,
        name: user.name.clone(),
        email: user.email.clone(),
    })
}

// Main function to start the Actix Web server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let pool = init_db_pool().await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/users", web::get().to(get_users))
            .route("/users", web::post().to(create_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
