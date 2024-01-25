use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use axum::{
    extract::{MatchedPath, State},
    http::Request,
    routing::{get, post},
    Form, Router,
};
use humantime::format_duration;
use maud::{html, Markup};
use serde::{Deserialize, Serialize};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{info, info_span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, Debug)]
struct AppState {
    users: Arc<Mutex<Vec<User>>>,
}

#[derive(Clone, Serialize, PartialEq, Deserialize, Debug)]
#[allow(dead_code)]
struct User {
    first_name: String,
    last_name: String,
    email: String,
}

#[tracing::instrument]
async fn post_form(State(state): State<AppState>, Form(user_form): Form<User>) -> Markup {
    let new_user = User {
        first_name: user_form.first_name,
        last_name: user_form.last_name,
        email: user_form.email,
    };

    let mut users = state.users.lock().expect("Mutex was poisoned !");
    users.push(new_user.clone());
    info!("Inserted {new_user:?}");

    html! {
        tr {
            td { (new_user.first_name) }
            td { (new_user.last_name) }
            td { (new_user.email) }
        }
    }
}

#[tracing::instrument]
async fn get_users(State(state): State<AppState>) -> Markup {
    let users = state.users.lock().expect("Mutex was poisoned !");
    html! {
    @for user in users.iter() {
           tr {
             td { (user.first_name) }
             td { (user.last_name) }
             td { (user.email) }
           }
            }
        }
}

#[tracing::instrument]
async fn healthcheck(time: SystemTime) -> Markup {
    match time.elapsed() {
        Ok(elapsed) => {
            html! {
                p { "Server uptime: "
                    span class="badge text-bg-primary"  {
                        (format_duration(Duration::from_secs(elapsed.as_secs())))
                    }
                    " secs"
                }
            }
        }
        Err(e) => html! {
            p { "Fatal error: " (e) }
        },
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "poc_htmx_rust=trace,tower_http=trace,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState {
        users: Arc::new(Mutex::new(Vec::new())),
    };

    let now: SystemTime = SystemTime::now();

    let template_routes = Router::new()
        .nest_service("/", ServeFile::new("templates/index.html"))
        .nest_service("/home", ServeFile::new("templates/home.html"))
        .nest_service("/get-form", ServeFile::new("templates/form.html"));

    let dir_routes = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/js", ServeDir::new("js"));

    let misc_routes =
        Router::new().nest_service("/favicon.ico", ServeFile::new("assets/favicon.ico"));

    let api_routes = Router::new()
        .route("/post-form", post(post_form))
        // By placing `layer` here it will be applied only on POST /post-form requests
        // and not on GET /healthcheck requests, avoiding log pollution.
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                info_span!(
                    "form_request",
                    method = ?request.method(),
                    matched_path,
                    some_other_field = tracing::field::Empty
                )
            }),
        )
        .route("/get-users", get(get_users))
        .with_state(state)
        .route("/healthcheck", get(move || healthcheck(now)));

    let app = Router::new()
        .merge(template_routes)
        .merge(dir_routes)
        .merge(misc_routes)
        .merge(api_routes);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
