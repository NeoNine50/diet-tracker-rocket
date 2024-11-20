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

#[derive(FromForm, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub gateway: f64,
    pub benefit: f64,
    pub time_created: String,
    pub resabundance: f64,
    pub beneficiaries: f64,
    pub producers: f64,
    pub ccs: f64,
    pub conssubben: f64,
    pub cco: f64,
    pub consobjben: f64,
    pub ceb: f64,
    pub envben: f64,
    pub chb: f64,
    pub humanben: f64,
    pub user_id: i64,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct TemplateMessage {
    is_user_product: bool,
    vec: Vec<Product>,
}

#[get("/product")]
pub fn addproduct_page() -> Template {
    Template::render("adduser", User { //this is intentional, lookup the template
        id: 0,
        name: String::new(),
        nbr: 0.0,
        fame: 0.0,
        time_created: String::new()
    })
}

#[get("/product/<product_id>")]
pub async fn product_page(product_id: i64, db_conn: &State<DbConn>) -> Template {
    let tmpconn = db_conn.lock().await;
    let product: User = tmpconn.query_row("SELECT id, name
    FROM products WHERE id = $1", [&product_id],
                                          |row| {
                                              Ok(User {
                                                  id: row.get(0)?,
                                                  name: row.get(1)?,
                                                  nbr: 0.0,
                                                  fame: 0.0,
                                                  time_created: String::new()
                                              })
                                          }).unwrap_or(User {
        id: 0,
        name: String::new(),
        nbr: 0.0,
        fame: 0.0,
        time_created: String::new()
    });

    Template::render("adduser", product)

}

#[get("/product/<product_id>/producers")]
pub async fn product_producers(product_id: i64, db_conn: &State<DbConn>) -> Template {
    let tmpconn = db_conn.lock().await;

    let mut vec = Vec::new();
    let mut stmt = tmpconn
        .prepare("SELECT users.id,
        users.name,
        user_products.gateway,
        user_products.benefit,
        user_products.time_created
        FROM user_products
        LEFT JOIN users ON users.id = user_products.UserID
        WHERE user_products.ProductID == $1
        ORDER BY name;")
        .unwrap();
    {
        let iter = stmt.query_map([&product_id], |row| {
            Ok(Product {
                id: row.get(0)?, //it has to go here because of template used
                name: row.get(1)?,
                gateway: row.get(2)?,
                benefit: row.get(3)?,
                time_created: row.get(4)?,
                resabundance: 0.0,
                beneficiaries: 0.0,
                producers: 0.0,
                ccs: 0.0,
                conssubben: 0.0,
                cco: 0.0,
                consobjben: 0.0,
                ceb: 0.0,
                envben: 0.0,
                chb: 0.0,
                humanben: 0.0,
                user_id: 0,
            })
        }).unwrap();
        for i in iter {
            vec.push(i.unwrap());
        }
    }

    Template::render("products", &TemplateMessage { is_user_product: true, vec})

}

#[post("/product", data = "<product>")]
pub async fn addproduct(product: Form<User>, db_conn: &State<DbConn>, templatedir: &State<TemplateDir>) -> Flash<Redirect> {
    let p = product.into_inner();
    let tmpconn = db_conn.lock().await;

    if p.id == 0 {
        tmpconn.execute("INSERT INTO products (name, gateway, benefit, time_created,
    resabundance, beneficiaries, producers, ccs, conssubben, cco, consobjben,
    ceb, envben, chb, humanben)
    VALUES ($1, $2, $3, datetime('now', 'localtime'), $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
                        params![&p.name, &0, &0,
                            &0, &0, &0, &0, &0, &0, &0,
                            &0, &0, &0, &0])
            .expect("insert single entry into products table");
        Flash::success(Redirect::to("/"),
                       if templatedir.0 { "Produkt přidán." } else { "Product added to items list." })
    }
    else {
        tmpconn.execute("UPDATE products SET name = $1, gateway = $2, benefit = $3,
    resabundance = $4, beneficiaries = $5, producers = $6, ccs = $7, conssubben = $8, cco = $9, consobjben = $10,
    ceb = $11, envben = $12, chb = $13, humanben = $14
    WHERE id = $15",
                        params![&p.name, &0, &0,
                            &0, &0, &0, &0, &0, &0, &0,
                            &0, &0, &0, &0, &p.id])
            .expect("update entry in products table");

        Flash::success(Redirect::to("/"),
                       if templatedir.0 { "Produkt upraven." } else { "Product modified." })
    }
}

#[get("/products")]
pub async fn products(db_conn: &State<DbConn>) -> Template {
    let tmpconn = db_conn.lock().await;
    let mut stmt = tmpconn
        .prepare("SELECT id, name, gateway, benefit, time_created FROM products ORDER BY name")
        .unwrap();

    let product_iter = stmt.query_map([], |row| {
        Ok(Product {
            id: row.get(0)?,
            name: row.get(1)?,
            gateway: row.get(2)?,
            benefit: row.get(3)?,
            time_created: row.get(4)?,
            resabundance: 0.0,
            beneficiaries: 0.0,
            producers: 0.0,
            ccs: 0.0,
            conssubben: 0.0,
            cco: 0.0,
            consobjben: 0.0,
            ceb: 0.0,
            envben: 0.0,
            chb: 0.0,
            humanben: 0.0,
            user_id: 0
        })
    }).unwrap();

    let mut vec = Vec::new();
    for product in product_iter {
        vec.push(product.unwrap());
    }

    Template::render("products", TemplateMessage { is_user_product: false, vec})
}
