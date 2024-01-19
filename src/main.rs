#[macro_use]
extern crate tera;
#[macro_use]
extern crate lazy_static;
extern crate serde_json;

use actix_files as fs;
use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use env_logger::Env;
use serde_json::value::{to_value, Value};
use std::collections::HashMap;
use std::error::Error;
use tera::{Context, Result, Tera};

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["html", ".sql"]);
        tera.register_filter("do_nothing", do_nothing_filter);
        tera
    };
}

pub fn do_nothing_filter(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let s = try_get_value!("do_nothing_filter", "value", String, value);
    Ok(to_value(s).unwrap())
}

fn get_arrow_coords(angle: u16) -> (f32, f32) {
    let rad = f32::from(angle).to_radians();
    (2.0*rad.sin(), -2.0*rad.cos())
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/sphere/{sphere_id}/{angle}")]
async fn sphere(path: web::Path<(String, u16)>) -> impl Responder {
    let (sphere_id, angle) = path.into_inner();
    let mut context = Context::new();

    context.insert("sphere_id", &sphere_id);
    context.insert("arrow_angle", &angle);
    context.insert("arrow_coords", &get_arrow_coords(angle));

    let template = TEMPLATES.render("sphere.html", &context).expect("Error!");

    HttpResponse::Ok().body(template)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(sphere)
            .service(fs::Files::new("/static/spheres", "assets/spheres/").show_files_listing())
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
    })
    .bind(("127.0.0.1", 7575))?
    .run()
    .await
}
