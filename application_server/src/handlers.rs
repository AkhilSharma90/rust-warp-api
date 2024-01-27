use crate::models::{OrderResponse, OrderItem, OrderRequestBody, Table, Menu, MenuResponse, TableResponse, OrderItemResponse};
use rusqlite::Connection;
use warp;
use rand::Rng;
use rusqlite::params;
use serde_json::json;


// Table Handlers

/// List All Tables
pub async fn list_table_handler(conn: Connection)-> Result<impl warp::Reply, warp::Rejection>{
    match Table::list(&conn) {
        Ok(tables) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&tables),
                warp::http::StatusCode::OK
            ))
        }
        Err(_err) => {
            Ok(warp::reply::with_status(
                warp::reply::json::<Vec<TableResponse>>(&vec![]),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            ))
        }
    }
}
/// Create a new Table
pub async fn create_table_handler(conn: Connection, data: Table) -> Result<impl warp::Reply, warp::Rejection> {
    match Table::get_existing_table_id(&conn, &data) {
    Ok(Some(table_id))=>{
        Ok(warp::reply::with_status(
            warp::reply::json(&json!({ "id": table_id })),
            warp::http::StatusCode::CREATED,
        ))
    }
    Ok(None)=>{
        match Table::create(&conn, &data) {
            Ok(table_id) => {
                Ok(warp::reply::with_status(
                    warp::reply::json(&json!({ "id": table_id })),
                    warp::http::StatusCode::CREATED,
                ))
            }
            Err(_err) => {
                // Respond with an error
                Ok(warp::reply::with_status(
                    warp::reply::json(&json!({"error":"Error creating table"})),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        }
    }
    Err(_err) => {
        // Respond with an error
        Ok(warp::reply::with_status(
            warp::reply::json(&json!({"error":"Error creating table"})),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}
    
}

// Menu Handler

/// List All Menus
pub async fn list_menu_handler(conn: Connection)-> Result<impl warp::Reply, warp::Rejection>{
    match Menu::list(&conn) {
        Ok(menus) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&menus),
                warp::http::StatusCode::OK,
            ))
        }
        Err(_err) => {
            Ok(warp::reply::with_status(
                warp::reply::json::<Vec<MenuResponse>>(&vec![]),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            )
        }
    }
}
// Create a new Menu
pub async fn create_menu_handler(conn: Connection, data: Menu) -> Result<impl warp::Reply, warp::Rejection> {
    match Menu::get_existing_menu_id(&conn, &data) {
        Ok(Some(menu_id))=>{
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({ "id": menu_id })),
                warp::http::StatusCode::CREATED,
            ))
        }
        Ok(None)=>{
            match Menu::create(&conn, &data) {
                Ok(menu_id) => {
                    Ok(warp::reply::with_status(
                        warp::reply::json(&json!({ "id": menu_id })),
                        warp::http::StatusCode::CREATED,
                    ))
                }
                Err(_err) => {
                    // Respond with an error
                    Ok(warp::reply::with_status(
                        warp::reply::json(&json!({ "error": "Error creating Menu" })),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        Err(_err) => {
            // Respond with an error
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({ "error": "Error creating Menu" })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
}
}



// Order Handlers

/// Create a new order
pub async fn create_order_handler(conn: Connection, req_body: OrderRequestBody) -> Result<impl warp::Reply, warp::Rejection> {
    let table_id = req_body.table_id;
    let menu_ids = req_body.menu_ids;
    if menu_ids.len() == 0{
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"error":"Please Add Items"})),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }
    // Check if there is an existing order with status 0 (running order) for the given table_id
    match OrderResponse::get_existing_order_id(&conn, table_id) {
        Ok(Some(order_id)) => {
            // Order exists for the given table_id, update the order items
            for menu_id in menu_ids {
                // Generate a random cooking time
                let cooking_time = rand::thread_rng().gen_range(5..=15);
                match OrderItem::get_existing_order_item_id(&conn, order_id, menu_id) {
                    Ok(Some(order_item_id)) => {
                         // Order item does exist, update quantity
                         match OrderItem::add_quantity_of_existing_order_item(&conn, order_item_id){
                            Ok(_)=>{
                                continue;
                            },
                            Err(_)=>{
                                return Ok(warp::reply::with_status(
                                    warp::reply::json(&json!({"error":"Error updating order Item"})),
                                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                                ));
                            }
                         }
                    }
                    Ok(None) => {
                        // Order item does not exist, create a new order item
                        match OrderItem::create(&conn, order_id, menu_id, cooking_time) {
                            Ok(_) => {
                                // Continue to the next menu_id
                                continue;
                            }
                            Err(_err) => {
                                // Return an error response
                                eprintln!("{}",_err);
                                return Ok(warp::reply::with_status(
                                    warp::reply::json(&json!({"error":"Error creating order Item"})),
                                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                                ));
                            }
                        }
                    }
                    Err(_err) => {
                        // Return an error response
                        return Ok(warp::reply::with_status(
                            warp::reply::json(&json!({"error":"Error creating for existing order Item"})),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        ));
                    }
                }
            }

            // If you reach this point, it means all order items were successfully handled
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({"success":"All order items updated successfully"})),
                warp::http::StatusCode::OK,
            ))
        }
        Ok(None) => {
            // No running order exists for the given table_id, create a new order and order items
            match OrderResponse::create(&conn, table_id) {
                Ok(last_inserted_id) => {
                    for menu_id in menu_ids {
                        // Generate a random cooking time
                        let cooking_time = rand::thread_rng().gen_range(5..=15);
                        match OrderItem::create(&conn, last_inserted_id, menu_id, cooking_time) {
                            Ok(_) => {
                                // Continue to the next menu_id
                                continue;
                            }
                            Err(_err) => {
                                // Return an error response
                                eprintln!("{}",_err);
                                return Ok(warp::reply::with_status(
                                    warp::reply::json(&json!({"error":"Error creating order Item"})),
                                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                                ));
                            }
                        }
                    }

                    Ok(warp::reply::with_status(
                        warp::reply::json(&json!({"id":last_inserted_id, "success":"Order and All Order Item Created Successfully"})),
                        warp::http::StatusCode::CREATED,
                    ))
                }
                Err(_err) => {
                    // Return an error response
                    Ok(warp::reply::with_status(
                        warp::reply::json(&json!({"error":format!("Error creating order {}", _err)})),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        Err(_err) => {
            // Return an error response
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error":"Error checking for existing order"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

/// List All Orders
pub async fn list_order_handler(conn: Connection)-> Result<impl warp::Reply, warp::Rejection>{
    match OrderResponse::list(&conn) {
        Ok(menus) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&menus),
                warp::http::StatusCode::OK,
            ))
        }
        Err(_err) => {
            Ok(
                warp::reply::with_status(
                warp::reply::json::<Vec<OrderResponse>>(&vec![]),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

/// Delete Specific Order Item from Order By Table
pub async fn delete_order_item_handler(conn: Connection, table_id: i64, menu_id: i64) -> Result<impl warp::Reply, warp::Rejection> {

    // Decrease the item quantity if greater than 1
    let result = conn.execute(
        "UPDATE order_items 
        SET cooking_time = cooking_time - (cooking_time/quantity), quantity = quantity - 1
        WHERE order_items.order_id IN (
            SELECT orders.id
            FROM orders
            JOIN tables ON orders.table_id = tables.id
            WHERE tables.id = ?1
        ) AND order_items.menu_id = ?2 AND order_items.quantity > 1",
        params![table_id, menu_id],
    );

    match result {
        Ok(updated) => {
            if updated > 0 {
                // If quantity was greater than 1, update and return success
                Ok(warp::reply::with_status(
                    warp::reply::json(&json!({"success": "Menu quantity updated successfully"})),
                    warp::http::StatusCode::OK,
                ))
            } else {
                // Quantity is 1, delete the order item
                let delete_result = conn.execute(
                    "DELETE FROM order_items 
                    WHERE order_items.order_id IN (
                        SELECT orders.id
                        FROM orders
                        JOIN tables ON orders.table_id = tables.id
                        WHERE tables.id = ?1
                    ) AND order_items.menu_id = ?2",
                    params![table_id, menu_id],
                );

                match delete_result {
                    Ok(_) => {
                        let order_id_result = OrderResponse::get_existing_order_id(&conn, table_id);

                        match order_id_result {
                            Ok(Some(order_id)) => {
                                let has_items = OrderResponse::has_items(&conn, order_id);

                                match has_items {
                                    Ok(false) => {
                                        // If there are no more items, delete the order as well
                                        let _ = conn.execute("DELETE from orders WHERE id = ?", params![order_id]);

                                        Ok(warp::reply::with_status(
                                            warp::reply::json(&json!({"success": "Menu deleted successfully and order deleted"})),
                                            warp::http::StatusCode::OK,
                                        ))
                                    }
                                    Ok(true)=>{
                                        Ok(warp::reply::with_status(
                                            warp::reply::json(&json!({"success": "Menu deleted successfully"})),
                                            warp::http::StatusCode::OK,
                                        )) 
                                    }
                                    Err(_err) => {
                                        Ok(warp::reply::with_status(
                                            warp::reply::json(&json!({"error": "Menu deleted failed"})),
                                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                                        ))
                                    }
                                }
                            }
                            _ => Ok(warp::reply::with_status(
                                warp::reply::json(&json!({"error": "Failed to retrieve order ID"})),
                                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                            )),
                        }
                    }
                    Err(_) => {
                        Ok(warp::reply::with_status(
                            warp::reply::json(&json!({"error": "Menu delete failed"})),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        ))
                    }
                }
            }
        }
        Err(_err) => {
            eprintln!("Failed to update quantity: {:?}", _err);
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error": "Failed to update quantity"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

/// List All Orders for a specific table
pub async fn list_order_items_for_table_handler(conn: Connection, table_id:i64)-> Result<impl warp::Reply, warp::Rejection>{
    match OrderItem::list_order_items(&conn, table_id) {
        Ok(items) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&items),
                warp::http::StatusCode::OK
            ))
        }
        Err(_err) => {
            eprintln!("{}", _err);
            Ok(warp::reply::with_status(
                warp::reply::json::<Vec<OrderItemResponse>>(&vec![]),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            ))
        }
    }
}

/// Retrieve a specific item from a specific table
pub async fn get_order_item_for_table_handler(conn: Connection, table_id:i64, menu_id: i64)-> Result<impl warp::Reply, warp::Rejection>{
    match OrderItem::get_item(&conn, table_id, menu_id) {
        Ok(Some(item)) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&item),
                warp::http::StatusCode::OK
            ))
        }
        Ok(None) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error": "No Item Found"})),
                warp::http::StatusCode::NOT_FOUND,
            ))
        }
        Err(_err) => {
            eprintln!("{}", _err);
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error": "Something Wrong!"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            ))
        }
    }
}


/// Unit Tests
#[cfg(test)]
mod tests {
    use warp::{Reply, hyper::Body};
    use super::*;


    // Set up the test database
    fn setup_test_db() -> Connection {
        println!("Initializing the test database...");
        let conn = Connection::open_in_memory().expect("Failed to create test database");
        conn.execute("PRAGMA foreign_keys = ON;", []).expect("Failed to enable foreign key support");
        conn.execute("CREATE TABLE IF NOT EXISTS tables (id INTEGER PRIMARY KEY,code TEXT NOT NULL UNIQUE)",[]).expect("Table table creation failed");
        conn.execute("CREATE TABLE IF NOT EXISTS menus (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",[]).expect("Menu table creation failed");
        conn.execute("CREATE TABLE IF NOT EXISTS orders (id INTEGER PRIMARY KEY, table_id INTEGER NOT NULL, FOREIGN KEY (table_id) REFERENCES tables(id), UNIQUE (table_id))",[]).expect("Order table creation failed");
        conn.execute("CREATE TABLE IF NOT EXISTS order_items (id INTEGER PRIMARY KEY, order_id INTEGER NOT NULL, menu_id INTEGER NOT NULL, cooking_time INTEGER NOT NULL,  quantity INTEGER NOT NULL default 1, FOREIGN KEY (order_id) REFERENCES orders(id), FOREIGN KEY (menu_id) REFERENCES menus(id))",[]).expect("OrderItems table creation failed");
        conn
    }

    // Inserting static table and menu data
    fn setup_static_data(conn: &Connection){
        let values_to_insert = vec!["T-01", "T-02", "T-03"];

        for value in values_to_insert {
            conn.execute("INSERT INTO tables (code) VALUES (?1)", &[value]).expect("Insertion Failed");
        }
        let values_to_insert = vec!["M-01", "M-02", "M-03", "M-04", "M-05"];

        for value in values_to_insert {
            conn.execute("INSERT INTO menus (name) VALUES (?1)", &[value]).expect("Insertion Failed");
        }

    }

    // Convert warp Response to serde Json Value
    async fn convert_response_to_json(resp:  warp::http::Response<Body>)->serde_json::Value {
        let body_bytes = warp::hyper::body::to_bytes(resp.into_body()).await.unwrap();
        let body_vec = body_bytes.to_vec();
        let body_string = String::from_utf8_lossy(&body_vec);
        let json_value: serde_json::Value = serde_json::from_str(&body_string).unwrap();
        return json_value;
    }

    // Test Case: 01 Menu Creation
    #[tokio::test]
    async fn test_create_menu_handler() {
        let conn = setup_test_db();
        let menu = Menu {
            id: 0,
            name: "Menu-01".to_string(),
        };
        let result = create_menu_handler(conn, menu).await;
        match result {
            Ok(rep)=>{
                let resp = rep.into_response();
                assert_eq!(resp.status(), warp::http::StatusCode::CREATED);
                let json_data = convert_response_to_json(resp).await;
                assert_eq!(json_data["id"].as_i64(), Some(1));
            }
            Err(_)=>{
                panic!("Unhandled Error");
            }
        }
    }

    // Test Case: 02 Table Creation
    #[tokio::test]
    async fn test_create_table_handler() {
        let conn = setup_test_db();
        let table = Table {
            id: 0,
            code: "Table-01".to_string(),
        };
        let result = create_table_handler(conn, table).await;
        match result {
            Ok(rep)=>{
                let resp = rep.into_response();
                assert_eq!(resp.status(), warp::http::StatusCode::CREATED);
                let json_data = convert_response_to_json(resp).await;
                assert_eq!(json_data["id"].as_i64(), Some(1));
            }
            Err(_)=>{
                panic!("Unhandled Error");
            }
        }
    }

    // Test Case: 03 Order creation fail with wrong data
    #[tokio::test]
    async fn test_create_order_handler_wrong_data() {
        let conn = setup_test_db();
        let order = OrderRequestBody {
            table_id: 1,
            menu_ids: vec![1, 2],
        };
        let result = create_order_handler(conn, order).await;
        // Will raise error, since table and menu not found
        match result {
            Ok(rep)=>{
                let resp = rep.into_response();
                assert_eq!(resp.status(), warp::http::StatusCode::INTERNAL_SERVER_ERROR);
                let json_data = convert_response_to_json(resp).await;
                assert_eq!(json_data["error"].as_str(), Some("Error creating order FOREIGN KEY constraint failed"));
            }
            Err(_)=>{
                panic!("Unhandled Error");
            }
        }
    }
    #[tokio::test]
    async fn test_create_order_handler_wrong_data2() {
        let mut conn = setup_test_db();
        setup_static_data(&mut conn);
        let order = OrderRequestBody {
            table_id: 1,
            menu_ids: vec![],
        };
        let result = create_order_handler(conn, order).await;
        // Will fail, since menu_ids empty
        match result {
            Ok(rep)=>{
                let resp = rep.into_response();
                assert_eq!(resp.status(), warp::http::StatusCode::BAD_REQUEST);
                let json_data = convert_response_to_json(resp).await;
                assert_eq!(json_data["error"].as_str(), Some("Please Add Items"));
            }
            Err(_)=>{
                panic!("Unhandled Error");
            }
        }
    }

    // Test Case: 04 Order creation with correct data
    #[tokio::test]
    async fn test_create_order_handler_correct_data(){
        let conn = setup_test_db();
        setup_static_data(&conn);
        let order = OrderRequestBody {
            table_id: 1,
            menu_ids: vec![1, 2],
        };

        let result = create_order_handler(conn, order).await;
        // Will create a new order for table_id 1 and menu 1, 2
        match result {
            Ok(rep)=>{
                let resp = rep.into_response();
                assert_eq!(resp.status(), warp::http::StatusCode::CREATED);
                let json_data = convert_response_to_json(resp).await;
                assert_eq!(json_data["id"].as_i64(), Some(1));
            }
            Err(_)=>{
                panic!("Unhandled Error");
            }
        }

    }

    // Test Case: 05 Remove Item From a Table
    #[tokio::test]
    async fn test_remove_item_from_table_handler(){
        let mut conn = setup_test_db();
        setup_static_data(&conn);
        // Start a transaction for creating order and order items
        let tx = conn.transaction().expect("Transaction Ceation Failed");

        // Insert into the orders table
        tx.execute(
            "INSERT INTO orders (table_id) VALUES (?1)",
            [1],
        ).expect("Order Creation Failed");

        // Get the last inserted order_id
        let order_id = tx.last_insert_rowid();

        // Insert into the order_items table using the obtained order_id
        tx.execute(
            "INSERT INTO order_items (order_id, menu_id, cooking_time) VALUES (?1, ?2, ?3)",
            [order_id, 1, 6],
        ).expect("OrderItems creation failed");

        tx.execute(
            "INSERT INTO order_items (order_id, menu_id, cooking_time) VALUES (?1, ?2, ?3)",
            [order_id, 2, 7],
        ).expect("OrderItems creation failed");

        // Commit the transaction
        tx.commit().expect("Commit Failed");
        let result = delete_order_item_handler(conn, 1, 2).await;
        // Will remove menu 2 from the order, menu 1 will be still there
        match result {
            Ok(rep)=>{
                let resp = rep.into_response();
                assert_eq!(resp.status(), warp::http::StatusCode::OK);
                let json_data = convert_response_to_json(resp).await;
                assert_eq!(json_data["success"].as_str(), Some("Menu deleted successfully"));
            }
            Err(_)=>{
                panic!("Unhandled Error");
            }
        }

    }

    // Test Case: 06 Removing all item from a order will delete the order
    #[tokio::test]
    async fn test_all_order_item_remove_handler(){
        let mut conn = setup_test_db();
        setup_static_data(&conn);
        // Start a transaction for creating order and order items
        let tx = conn.transaction().expect("Transaction Ceation Failed");

        // Insert into the orders table
        tx.execute(
            "INSERT INTO orders (table_id) VALUES (?1)",
            [1],
        ).expect("Order Creation Failed");

        // Get the last inserted order_id
        let order_id = tx.last_insert_rowid();

        // Insert into the order_items table using the obtained order_id
        tx.execute(
            "INSERT INTO order_items (order_id, menu_id, cooking_time) VALUES (?1, ?2, ?3)",
            [order_id, 1, 6],
        ).expect("OrderItems creation failed");

        // Commit the transaction
        tx.commit().expect("Commit Failed");
        let result = delete_order_item_handler(conn, 1, 1).await;
        // Will remove menu 1 from the order, and since no item i order, order will be deleted
        match result {
            Ok(rep)=>{
                let resp = rep.into_response();
                assert_eq!(resp.status(), warp::http::StatusCode::OK);
                let json_data = convert_response_to_json(resp).await;
                assert_eq!(json_data["success"].as_str(), Some("Menu deleted successfully and order deleted"));
            }
            Err(_)=>{
                panic!("Unhandled Error");
            }
        }

    }

    // Test Case: 07 Removing item having quantity more than 1 will reduce the quantity of the item
    #[tokio::test]
    async fn test_order_item_quantity_reduce_handler(){
        let mut conn = setup_test_db();
        setup_static_data(&conn);
        // Start a transaction for creating order and order items
        let tx = conn.transaction().expect("Transaction Ceation Failed");

        // Insert into the orders table
        tx.execute(
            "INSERT INTO orders (table_id) VALUES (?1)",
            [1],
        ).expect("Order Creation Failed");

        // Get the last inserted order_id
        let order_id = tx.last_insert_rowid();

        // Insert into the order_items table using the obtained order_id
        tx.execute(
            "INSERT INTO order_items (order_id, menu_id, cooking_time, quantity) VALUES (?1, ?2, ?3, ?4)",
            [order_id, 1, 6, 2],
        ).expect("OrderItems creation failed");

        // Commit the transaction
        tx.commit().expect("Commit Failed");
        let result = delete_order_item_handler(conn, 1, 1).await;
        // Will update the quantity of menu 1
        match result {
            Ok(rep)=>{
                let resp = rep.into_response();
                assert_eq!(resp.status(), warp::http::StatusCode::OK);
                let json_data = convert_response_to_json(resp).await;
                assert_eq!(json_data["success"].as_str(), Some("Menu quantity updated successfully"));
            }
            Err(_)=>{
                panic!("Unhandled Error");
            }
        }

    }

     // Test Case: 08 Get Specific Item from a Table
    #[tokio::test]
    async fn test_get_item_from_table_handler(){
        let mut conn = setup_test_db();
        setup_static_data(&conn);
        // Start a transaction for creating order and order items
        let tx = conn.transaction().expect("Transaction Ceation Failed");

        // Insert into the orders table
        tx.execute(
            "INSERT INTO orders (table_id) VALUES (?1)",
            [1],
        ).expect("Order Creation Failed");

        // Get the last inserted order_id
        let order_id = tx.last_insert_rowid();

        // Insert into the order_items table using the obtained order_id
        tx.execute(
            "INSERT INTO order_items (order_id, menu_id, cooking_time) VALUES (?1, ?2, ?3)",
            [order_id, 1, 6],
        ).expect("OrderItems creation failed");

        tx.execute(
            "INSERT INTO order_items (order_id, menu_id, cooking_time) VALUES (?1, ?2, ?3)",
            [order_id, 2, 7],
        ).expect("OrderItems creation failed");

        // Commit the transaction
        tx.commit().expect("Commit Failed");

        let result = get_order_item_for_table_handler(conn, 1, 2).await;
        // Will retrieve menu 2 from the table
        match result {
            Ok(rep)=>{
                let resp = rep.into_response();
                match resp.status() {
                    // If item found, get item
                    warp::http::StatusCode::OK=>{
                        let json_data = convert_response_to_json(resp).await;
                        assert_eq!(json_data["menu_name"].as_str(), Some("M-02"));
                    },
                    // If item not found raise NotFound
                    warp::http::StatusCode::NOT_FOUND=>{
                        let json_data = convert_response_to_json(resp).await;
                        assert_eq!(json_data["error"].as_str(), Some("No Item Found"));
                    },
                    _ => {}
                }
            }
            Err(_)=>{
                panic!("Unhandled Error");
            }
        }

    }
}