use axum::{
    routing::{get, post},
    Router,
};
use maud::{html, Markup};

async fn hello_world() -> Markup {
    html! {
        h1 { "Hello, World!" }
    }
}

async fn clicked() -> Markup {
    html! {
        p { "You clicked !"}
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/clicked", post(clicked));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
