// src/main.rs
mod models;
mod handlers;
mod db;
mod routes;
use warp::Filter;

#[tokio::main]
async fn main() {
    // Initialize DB
    db::initialize_db();
    
    // Combine all routes
    let routes = routes::restaurent_routes();

    // Start the warp server
    println!("Running the server");
    warp::serve(routes.with(warp::trace::request()))
        .run(([127, 0, 0, 1], 3030))
        .await;
}