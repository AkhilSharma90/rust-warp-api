// src/routes.rs
use crate::handlers::{
    create_order_handler,
    list_table_handler,
    create_table_handler,
    list_menu_handler,
    create_menu_handler,
    list_order_handler,
    delete_order_item_handler,
    list_order_items_for_table_handler,
    get_order_item_for_table_handler
};
use warp::{Filter, Rejection, Reply};
use rusqlite::Connection;
use crate::db::get_db_conn;
use std::convert::Infallible;

/// Middleware to handle errors and convert them into a JSON response
/// For now it handles Route Not Found and Deserialization Error.
/// We can add custom action to handle different type of error
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {

    // If route not found
    if err.is_not_found() {
        Ok(warp::reply::with_status(
            warp::reply::json(&format!("Mahadi Error: {:?}", err)),
            warp::http::StatusCode::NOT_FOUND,
        ))
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        // If fail to deserialize request body
        Ok(warp::reply::with_status(
            warp::reply::json(&format!("Error: Failed to deserialize request body")),
            warp::http::StatusCode::BAD_REQUEST,
        ))
    } else {
        // Default to Internal Server Error for other errors
        Ok(warp::reply::with_status(
            warp::reply::json(&format!("Error: {:?}", err)),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

/// Helper function to provide a database connection to route handlers
/// Returns a New Db connection Per Route
fn with_db() -> impl Filter<Extract = (Connection,), Error = Infallible> + Clone {
    warp::any().map(|| get_db_conn())
}

/// This Route lists all orders. GET request
pub fn list_all_orders_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    return warp::path!("orders")
        .and(warp::get())
        .and(with_db())
        .and_then(|conn| list_order_handler(conn))
}


/// This Route creates a new order
/// Its a POST request and expects table_id: i64 and menu_ids: vec![i64]
/// If menu_ids is empty, return BAD REQUEST
/// If there is already existing order (status=0) for this table_id, try to add new items t the existing order. Return success or error message
/// If no exisiting order or order with (status=1), creates a new order and return id
pub fn create_order_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    return warp::path!("orders"/"create")
        .and(warp::post())
        .and(with_db())
        .and(warp::body::json())
        .and_then(|conn, req_body| create_order_handler(conn, req_body))
        
}

/// This Route to delete specific menu from table.
/// Its a delete request. /orders/{table_id}/items/{item_id}
/// If item found for this table, deleted the item and return success/error message 
/// If this is the las item in this table, update order status=1 marking it as complete
pub fn delete_item_from_order_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    return warp::path!("orders"/i64/"items"/i64)
        .and(warp::delete())
        .and(with_db())
        .and_then(|table_id, menu_id, conn| delete_order_item_handler(conn, table_id, menu_id))
        
}

/// This Route lists all tables
pub fn list_tables_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    return warp::path!("tables")
        .and(warp::get())
        .and(with_db())
        .and_then(|conn| list_table_handler(conn))
}

/// This Route creates a table.
/// It expects a code in the request POST body. Returns id on successfull creation
pub fn create_table_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    return warp::path!("tables"/"create")
        .and(warp::post())
        .and(with_db())
        .and(warp::body::json())
        .and_then(|conn, req_body| create_table_handler(conn, req_body))
}

/// This Route lists all menus for a table. /tables/{table_id}/items
pub fn list_order_items_for_table_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    return warp::path!("tables"/i64/"items")
        .and(warp::get())
        .and(with_db())
        .and_then(|table_id, conn| list_order_items_for_table_handler(conn, table_id))
}

/// This Route retrieves a specific menu for table. /tables/{table_id}/items/{item_id}
pub fn get_item_from_order_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    return warp::path!("tables"/i64/"items"/i64)
        .and(warp::get())
        .and(with_db())
        .and_then(|table_id, menu_id, conn| get_order_item_for_table_handler(conn, table_id, menu_id))
        
}

/// This Route lists all menus
pub fn list_menus_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    return warp::path!("menus")
        .and(warp::get())
        .and(with_db())
        .and_then(|conn| list_menu_handler(conn))
        
}

///  This Route creates a menu
/// It expects a name in request POST body
pub fn create_menu_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    return warp::path!("menus"/"create")
        .and(warp::post())
        .and(with_db())
        .and(warp::body::json())
        .and_then(|conn, req_body| create_menu_handler(conn, req_body))
}

/// Combine all routes
pub fn restaurent_routes()->impl Filter<Extract = impl Reply, Error = Rejection> + Clone{
    let routes = create_order_route()
    .or(create_table_route())
    .or(create_menu_route())
    .or(list_tables_route())
    .or(list_menus_route())
    .or(list_all_orders_route())
    .or(delete_item_from_order_route())
    .or(list_order_items_for_table_route())
    .or(get_item_from_order_route());

    routes.recover(handle_rejection)
}