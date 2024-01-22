use axum::{routing::post, Form, Router};
use maud::{html, Markup};
use serde::Deserialize;
use tower_http::services::{ServeDir, ServeFile};

async fn clicked() -> Markup {
    html! {
        p { "You clicked !"}
    }
}

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

#[tokio::main]
async fn main() {
    let template_routes = Router::new()
        .nest_service("/", ServeFile::new("templates/index.html"))
        .nest_service("/get-form", ServeFile::new("templates/form.html"));

    let dir_routes = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/js", ServeDir::new("js"));

    let misc_routes =
        Router::new().nest_service("/favicon.ico", ServeFile::new("assets/favicon.ico"));

    let api_routes = Router::new()
        .route("/clicked", post(clicked))
        .route("/post-form", post(post_form));

    let app = Router::new()
        .merge(template_routes)
        .merge(dir_routes)
        .merge(misc_routes)
        .merge(api_routes);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
