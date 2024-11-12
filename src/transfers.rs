//use std::sync::Mutex;
use rocket_sync_db_pools::rusqlite::params;
use rocket::form::Form;
use rocket_dyn_templates::Template;
use rocket::State;
use rocket::response::Redirect;
//use rocket::request::FlashMessage;
use rocket::response::Flash;
use rocket::serde::Serialize;

use crate::users::User;
use super::{DbConn,TemplateDir};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ContextTransfer {
    users: Vec<User>,
    products: Vec<User>,
}

#[derive(FromForm)]
pub struct Transfer {
    producer: i64,
    consumer: i64,
    product: i64,
    amount: f64,
    comment: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct NamedTransfer {
    id: i64,
    producer: String,
    consumer: String,
    product: String,
    amount: f64,
    nbr: f64,
    time_created: String,
    gnbr: f64,
    comment: String,
}

#[get("/transfer")]
pub async fn transfer_page(conn: &State<DbConn> ) -> Template {
    let tmpconn = conn.lock().await;

    let mut users = Vec::new();
    let mut stmt = tmpconn
        .prepare("SELECT id, name, NBR FROM users WHERE id != 0 ORDER BY name")
        .unwrap();
    {
        let user_iter = stmt.query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                nbr: row.get(2)?,
                fame: 0.0,
                time_created: String::new(),
            })
        }).unwrap();
        for user in user_iter {
            users.push(user.unwrap());
        }
    }
    stmt = tmpconn
        .prepare("SELECT id, name FROM products
        WHERE id IN (SELECT ProductID FROM user_products)
        ORDER BY name")
        .unwrap();
    let item_iter = stmt.query_map([], |row| {
        Ok(User { //yes, this is on purpose to save data
            id: row.get(0)?,
            name: row.get(1)?,
            nbr: 0.0,
            fame: 0.0,
            time_created: String::new(),
        })
    }).unwrap();
    let mut products = Vec::new();
    for item in item_iter {
        products.push(item.unwrap());
    }

    let context: ContextTransfer = ContextTransfer {
        users,
        products,
        //nbr: 10
    };

    Template::render("transfer", &context)
}

#[post("/transfer", data = "<post>")]
pub async fn transfer(conn: &State<DbConn>, post: Form<Transfer>, templatedir: &State<TemplateDir>) -> Flash<Redirect> {
    let transfer = post.into_inner();

    let tmpconn = conn.lock().await;

    // Check if the product exists for the producer
    let product_query = tmpconn.query_row(
        "SELECT gateway, benefit FROM user_products WHERE ProductID = $1 AND UserID = $2",
        [&transfer.product, &transfer.producer],
        |row| Ok((row.get(0)?, row.get(1)?))
    );

    if product_query.is_err() {
        return Flash::success(
            Redirect::to("/"),
            if templatedir.0 {
                "Produkt musí být nejprve uživateli přiřazen."
            } else {
                "Product must be assigned to profile first. Click your browser's back to previous page button and click 'Assign sensitivity and values'."
            },
        );
    }

    let product_params: (f64, f64) = product_query.unwrap();

    // Insert transfer record in the transfers table
    tmpconn.execute(
        "INSERT INTO transfers (ProducerID, ConsumerID, ProductID, amount, NBR, GNBR, comment, time_created) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, datetime('now', 'localtime'))",
        params![
            &transfer.producer,
            &transfer.consumer,
            &transfer.product,
            &transfer.amount,
            &(product_params.1 * transfer.amount),
            &(product_params.0 * transfer.amount),
            if transfer.comment.is_empty() { None } else { Some(&transfer.comment) }
        ]
    ).expect("insert single entry into transfers table");

    // Update compliance score
    tmpconn.execute(
        "UPDATE users SET NBR = NBR + $1, fame = fame + $1 WHERE id = $2",
        params![&(product_params.1 * transfer.amount), &transfer.producer]
    ).expect("update entries in users table");

    // If producer and consumer are different, update NBR for consumer
    if transfer.producer != transfer.consumer {
        tmpconn.execute(
            "UPDATE users SET NBR = NBR - $1 WHERE id = $2",
            params![&(product_params.0 * transfer.amount), &transfer.consumer]
        ).expect("update entries in users table");
    }

    Flash::success(
        Redirect::to("/"),
        if templatedir.0 {
            "Transfer proveden."
        } else {
            "Transfer complete."
        },
    )
}

#[post("/updatetransfer/<transfer_id>", data = "<updated_comment>")]
pub async fn update_transfer_comment(conn: &State<DbConn>, transfer_id: i64, updated_comment: Form<String>) -> Flash<Redirect> {
    let tmpconn = conn.lock().await;

    let comment = updated_comment.into_inner();
    tmpconn.execute(
        "UPDATE transfers SET comment = $1 WHERE id = $2",
        params![&comment, &transfer_id],
    ).expect("Failed to update comment.");

    Flash::success(Redirect::to("http://localhost:8000"), "Note updated successfully.")
}

#[get("/transfers")]
pub async fn transfers(conn: &State<DbConn>) -> Template {
    let tmpconn = conn.lock().await;

    let mut transfers = Vec::new();
    let mut stmt = tmpconn
        .prepare("SELECT t2.id          AS table2_id
        , t2.ConsumerID
        , t2.ProducerID
        , t31.name      AS x
        , t32.name      AS y
        , p.name		 AS z
	    , t2.amount
	    , t2.NBR
        , t2.time_created
        , t2.GNBR
        , t2.comment
        FROM   transfers t2
        LEFT   JOIN users t31 ON t31.id = t2.ConsumerID
        LEFT   JOIN users t32 ON t32.id = t2.ProducerID
        LEFT   JOIN products p ON p.id = t2.ProductID
        ORDER BY t2.time_created DESC;")
        .unwrap();
    {
        let transfer_iter = stmt.query_map([], |row| {
            Ok(NamedTransfer {
                id: row.get(0)?,
                producer: row.get(4)?,
                consumer: row.get(3)?,
                product: row.get(5)?,
                amount: row.get(6)?,
                nbr: row.get(7)?,
                time_created: row.get(8)?,
                gnbr: row.get(9)?,
                comment: row.get(10).unwrap_or(String::new()),
            })
        }).unwrap();
        for transfer in transfer_iter {
            transfers.push(transfer.unwrap());
        }
    }

    Template::render("transfers", &transfers)
}

#[get("/deletetransfer/<transfer_id>")]
pub async fn delete_transfer(conn: &State<DbConn>, transfer_id: i64, templatedir: &State<TemplateDir>) -> Flash<Redirect> {

    let tmpconn = conn.lock().await;

    let transfer_params: (i64, i64, f64, f64) = tmpconn.query_row(
        "SELECT ProducerID, ConsumerID, NBR, GNBR FROM transfers WHERE id = $1",
        [&transfer_id], |row| { Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)) })
        .expect("product does not exist");


    tmpconn.execute("DELETE FROM transfers WHERE id = $1",
                    params![&transfer_id])
        .expect("delete single entry from transfers table");
    tmpconn.execute("UPDATE users SET NBR = NBR - $1, fame = fame - $1 WHERE id = $2",
                    params![&transfer_params.2, &transfer_params.0])
        .expect("update entries in users table");
    tmpconn.execute("UPDATE users SET NBR = NBR + $1 WHERE id = $2",
                    params![&transfer_params.3, &transfer_params.1])
        .expect("update entries in users table");

    Flash::success(Redirect::to("/"),
                   if templatedir.0 { "Transfer smazán." }
                       else { "Transfer deleted." })
}
