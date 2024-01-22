use std::time::SystemTime;

use axum::{
    routing::{get, post},
    Form, Router,
};
use maud::{html, Markup};
use serde::Deserialize;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct MyForm {
    first_name: String,
    last_name: String,
    email: String,
}

async fn post_form(Form(form): Form<MyForm>) -> Markup {
    html! {
        p { "Hi Mr " (form.first_name) " " (form.last_name) " !" }
    }
}

async fn healthcheck(time: SystemTime) -> Markup {
    match time.elapsed() {
        Ok(elapsed) => {
            html! {
                p { "server uptime: " (elapsed.as_secs()) " secs" }
            }
        }
        Err(e) => html! {
            p { "Fatal error: " (e) }
        },
    }
}

#[tokio::main]
async fn main() {
    let now: SystemTime = SystemTime::now();

    let template_routes = Router::new()
        .nest_service("/", ServeFile::new("templates/index.html"))
        .nest_service("/get-form", ServeFile::new("templates/form.html"));

    let dir_routes = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/js", ServeDir::new("js"));

    let misc_routes =
        Router::new().nest_service("/favicon.ico", ServeFile::new("assets/favicon.ico"));

    let api_routes = Router::new()
        .route("/post-form", post(post_form))
        .route("/healthcheck", get(move || healthcheck(now)));

    let app = Router::new()
        .merge(template_routes)
        .merge(dir_routes)
        .merge(misc_routes)
        .merge(api_routes);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
