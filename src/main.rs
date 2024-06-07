use std::{
    fmt, fs,
    str::FromStr,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use axum::{
    extract::{MatchedPath, Query, State},
    http::Request,
    response::{IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use humantime::format_duration;
use maud::{html, Markup, PreEscaped};
use serde::{de, Deserialize, Deserializer, Serialize};
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
async fn post_form(State(state): State<AppState>, Form(user_form): Form<User>) -> Response {
    let new_user = User {
        first_name: user_form.first_name,
        last_name: user_form.last_name,
        email: user_form.email,
    };

    let mut users = state.users.lock().expect("Mutex was poisoned !");
    users.push(new_user.clone());
    info!("Inserted {new_user:?}");

    let body = html! {
        tr {
            td { (new_user.first_name) }
            td { (new_user.last_name) }
            td { (new_user.email) }
        }
    };

    ([("HX-Trigger-After-Swap", "newUser")], body).into_response()
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

fn breadcrumb_inactive_item(item: &str, target_step: &str) -> Markup {
    html! {
        li class="breadcrumb-item" {
            a href="#" hx-get=("/get-".to_owned() + target_step) hx-target="#breadcrumb-container" { (item) }
        }
    }
}

fn breadcrumb_active_item(item: &str) -> Markup {
    html! {
        li class="breadcrumb-item active" aria-current="page" { (item) }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Step {
    Step1,
    Step2,
    Step3,
}

impl ToString for Step {
    fn to_string(&self) -> String {
        match self {
            Step::Step1 => "step1".to_owned(),
            Step::Step2 => "step2".to_owned(),
            Step::Step3 => "step3".to_owned(),
        }
    }
}

fn breadcrumb_header(current_step: &Step) -> Markup {
    html! {
        nav style="--bs-breadcrumb-divider: '>';" aria-label="breadcrumb" {
            ol class="breadcrumb" {
                (if current_step == &Step::Step1 {
                    breadcrumb_active_item("Step 1")
                } else {
                    breadcrumb_inactive_item("Step 1", "step1")
                })
                (if current_step == &Step::Step2 {
                    breadcrumb_active_item("Step 2")
                } else {
                    breadcrumb_inactive_item("Step 2", "step2")
                })
                (if current_step == &Step::Step3 {
                    breadcrumb_active_item("Step 3")
                } else {
                    breadcrumb_inactive_item("Step 3", "step3")
                })
            }
        }
    }
}

fn breadcrumb_footer(previous_step: Option<String>, next_step: Option<String>) -> Markup {
    html! {
        div id="breadcrumb-footer" {
            @match previous_step {
                Some(step) => {
                    a class="btn btn-primary" href="#"
                        hx-get=("/get-".to_owned() + step.as_ref())
                        hx-target="#breadcrumb-container"
                        { "Previous" }
                },
                None => {}
            }
            @match next_step {
                Some(step) => {
                    a class="btn btn-primary" href="#"
                        hx-get=("/get-".to_owned() + step.as_ref())
                        hx-target="#breadcrumb-container"
                        { "Next" }
                },
                None => {}
            }
        }
    }
}

#[tracing::instrument]
async fn get_step(step: &Step, step_content: String) -> Markup {
    html! {
       (breadcrumb_header(step))
       div id="breadcrumb-content" {
           (PreEscaped(step_content))
       }
       (breadcrumb_footer(match step {
           Step::Step1 => None,
           Step::Step2 => Some(Step::Step1.to_string()),
           Step::Step3 => Some(Step::Step2.to_string()),
       }, match step {
           Step::Step1 => Some(Step::Step2.to_string()),
           Step::Step2 => Some(Step::Step3.to_string()),
           Step::Step3 => None,
       }))
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

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SnackbarParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    first_name: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    last_name: Option<String>,
}

/// Serde deserialization decorator to map empty Strings to None,
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

#[tracing::instrument]
async fn get_new_user_snackbar(Query(params): Query<SnackbarParams>) -> Markup {
    match (params.first_name, params.last_name) {
        (Some(f), Some(l)) => html! {
            div class="alert alert-success alert-dismissible fade show" role="alert" {
                (format!("L'utilisateur {f} {l} a bien été ajouté !"))
                button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close" {
                }
            }
        },
        _ => html! {
            div class="alert alert-danger alert-dismissible fade show" role="alert" {
                (format!("Une erreur est survenue pendant l'ajout de l'utilisateur !"))
                button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close" {
                }
            }
        },
    }
}
// (user.first_name.to_owned() + " " + user.last_name.as_ref() + " added !")

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
        .nest_service("/get-form", ServeFile::new("templates/form.html"))
        .nest_service(
            "/get-breadcrumb",
            ServeFile::new("templates/breadcrumb.html"),
        );

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
        .route("/get-new-user-snackbar", get(get_new_user_snackbar))
        .with_state(state)
        .route(
            "/get-step1",
            get(|| {
                get_step(
                    &Step::Step1,
                    fs::read_to_string("templates/step1.html").expect("Couldn't read step1.html"),
                )
            }),
        )
        .route(
            "/get-step2",
            get(|| {
                get_step(
                    &Step::Step2,
                    fs::read_to_string("templates/step2.html").expect("Couldn't read step2.html"),
                )
            }),
        )
        .route(
            "/get-step3",
            get(|| {
                get_step(
                    &Step::Step3,
                    fs::read_to_string("templates/step3.html").expect("Couldn't read step3.html"),
                )
            }),
        )
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
