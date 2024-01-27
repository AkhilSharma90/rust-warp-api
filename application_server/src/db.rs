use rusqlite::Connection;

pub fn get_db_conn()->Connection{
    let conn = Connection::open("restaurent.db").expect("Failed to open SQLite connection");
    conn
}
pub fn initialize_db() {
    println!("Initializing the database...");
    let conn = Connection::open("restaurent.db").expect("Failed to open SQLite connection");
    //Enable Foreignkey support
    conn.execute("PRAGMA foreign_keys = ON;", []).expect("Failed to enable foreign key support");
    println!("Creating Table table");
    create_table_table_if_not_exists(&conn).expect("Failed to create Table table");
    println!("Creating Menu table");
    create_menu_table_if_not_exists(&conn).expect("Failed to create Table menus");
    println!("Creating Order table");
    create_order_table_if_not_exists(&conn).expect("Failed to create Table orders");
    println!("Creating OrderItem table");
    create_order_item_table_if_not_exists(&conn).expect("Failed to create Table order_items");
}

fn create_table_table_if_not_exists(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("CREATE TABLE IF NOT EXISTS tables (id INTEGER PRIMARY KEY,code TEXT NOT NULL UNIQUE)",[])?;
    Ok(())
}
fn create_menu_table_if_not_exists(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("CREATE TABLE IF NOT EXISTS menus (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",[])?;
    Ok(())
}
fn create_order_table_if_not_exists(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("CREATE TABLE IF NOT EXISTS orders (id INTEGER PRIMARY KEY, table_id INTEGER NOT NULL, FOREIGN KEY (table_id) REFERENCES tables(id), UNIQUE (table_id))",[])?;
    Ok(())
}
fn create_order_item_table_if_not_exists(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("CREATE TABLE IF NOT EXISTS order_items (id INTEGER PRIMARY KEY, order_id INTEGER NOT NULL, menu_id INTEGER NOT NULL, cooking_time INTEGER NOT NULL, quantity INTEGER NOT NULL default 1, FOREIGN KEY (order_id) REFERENCES orders(id), FOREIGN KEY (menu_id) REFERENCES menus(id))",[])?;
    Ok(())
}

