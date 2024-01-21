use actix_files as fs;
use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use env_logger::Env;
use lazy_static::lazy_static;
use serde_json::value::{to_value, Value};
use std::collections::HashMap;
use tera::{try_get_value, Context, Result, Tera};

struct AppState {
    sphere_data: toml::Table,
}

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
    (1.0 * rad.sin(), -1.0 * rad.cos())
}

#[get("/")]
async fn hello(_data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/sphere")]
async fn sphere(_data: web::Data<AppState>) -> impl Responder {
    let mut context = Context::new();

    let template = TEMPLATES.render("sphere.html", &context).expect("Error!");

    HttpResponse::Ok().body(template)
}

#[get("/api/sphere_data/{sphere_id}")]
async fn api_sphere_data(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let sphere_id = path.into_inner();

    let mut sphere_data_list: Vec<&toml::Value> = data.sphere_data["spheres"]
        .as_array()
        .unwrap_or_else(|| panic!("Expected an array"))
        .iter()
        .filter(|sp| sp["id"].as_str() == Some(sphere_id.as_str()))
        .collect();

    match sphere_data_list.first_mut() {
        Some(sphere_data) => {
            let mut resp_data = sphere_data.clone();
            
            for i in 0..resp_data["neighbours"].as_array().expect("").len() {
                if let Some(n_data) = resp_data["neighbours"][i].as_table_mut() {
                    let (x, y) = get_arrow_coords(n_data["angle"].clone().try_into::<u16>().unwrap());
                    
                    let coord_data = toml::Value::Table([
                        ("x".to_string(), toml::Value::Float(x as f64)),
                        ("y".to_string(), toml::Value::Float(y as f64))
                    ].iter().cloned().collect());
                    n_data.insert("coordinates".to_string(), coord_data);
                    println!("{}", n_data["id"]);
                }
            }

            HttpResponse::Ok().json(resp_data)
        },
        None => HttpResponse::BadRequest()
            .content_type("application/json")
            .body("{\"Error\": \"Invalid sphere_id\"}"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let path = std::path::Path::new("src/spheres.toml");
    let file = std::fs::read_to_string(path)?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                sphere_data: file.parse().unwrap(),
            }))
            .service(hello)
            .service(sphere)
            .service(api_sphere_data)
            .service(fs::Files::new("/static/", "assets/").show_files_listing())
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
    })
    .bind(("127.0.0.1", 7575))?
    .run()
    .await
}
