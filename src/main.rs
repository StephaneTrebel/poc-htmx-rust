use axum::{routing::post, Router};
use maud::{html, Markup};
use tower_http::services::{ServeDir, ServeFile};

async fn clicked() -> Markup {
    html! {
        p { "You clicked !"}
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

    let api_routes = Router::new().route("/clicked", post(clicked));

    let app = Router::new()
        .merge(template_routes)
        .merge(dir_routes)
        .merge(misc_routes)
        .merge(api_routes);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
