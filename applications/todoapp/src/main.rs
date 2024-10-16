use actix_files::Files; // Import actix-files to serve static files
use actix_web::{
    delete, get, middleware::Logger, post, put, web, App, HttpResponse, HttpServer, Responder,
};
use dotenv::dotenv;
use lazy_static::lazy_static;
use prometheus::{register_int_counter, Encoder, IntCounter, TextEncoder};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::env;

#[derive(Serialize, Deserialize)]
struct Todo {
    id: i32,
    title: String,
    completed: bool,
}

#[derive(Deserialize)]
struct CreateTodo {
    title: String,
}

#[derive(Deserialize)]
struct UpdateTodo {
    completed: bool,
}

// Metrics
lazy_static! {
    static ref TODO_CREATED: IntCounter =
        register_int_counter!("todo_created_total", "Total number of created todo items").unwrap();
    static ref TODO_COMPLETED: IntCounter = register_int_counter!(
        "todo_completed_total",
        "Total number of completed todo items"
    )
    .unwrap();
}

// Get all todos
#[get("/todos")]
async fn get_todos(pool: web::Data<sqlx::PgPool>) -> impl Responder {
    let todos = sqlx::query_as!(Todo, "SELECT id, title, completed FROM todos")
        .fetch_all(pool.get_ref())
        .await;

    match todos {
        Ok(todos) => HttpResponse::Ok().json(todos),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Create a new todo
#[post("/todos")]
async fn create_todo(pool: web::Data<sqlx::PgPool>, item: web::Json<CreateTodo>) -> impl Responder {
    let result = sqlx::query!(
        "INSERT INTO todos (title, completed) VALUES ($1, $2) RETURNING id",
        item.title,
        false
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(record) => {
            TODO_CREATED.inc();
            HttpResponse::Ok().json(Todo {
                id: record.id,
                title: item.title.clone(),
                completed: false,
            })
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Update a todo
#[put("/todos/{id}")]
async fn update_todo(
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<i32>,
    item: web::Json<UpdateTodo>,
) -> impl Responder {
    let id = path.into_inner();
    let result = sqlx::query!(
        "UPDATE todos SET completed = $1 WHERE id = $2",
        item.completed,
        id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => {
            if item.completed {
                TODO_COMPLETED.inc();
            }
            HttpResponse::Ok().finish()
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Delete a todo
#[delete("/todos/{id}")]
async fn delete_todo(pool: web::Data<sqlx::PgPool>, path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();
    let result = sqlx::query!("DELETE FROM todos WHERE id = $1", id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Metrics endpoint
#[get("/metrics")]
async fn metrics() -> impl Responder {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();

    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    HttpResponse::Ok()
        .content_type(encoder.format_type())
        .body(buffer)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .service(get_todos)
            .service(create_todo)
            .service(update_todo)
            .service(delete_todo)
            .service(metrics)
            .service(Files::new("/", "./static").index_file("index.html")) // Serve static files
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
