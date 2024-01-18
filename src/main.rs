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
    let app = Router::new()
        .nest_service(
            "/",
            ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html")),
        )
        .route("/clicked", post(clicked));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
