mod db;
mod eventbus;

use std::{
    path::PathBuf,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
    usize,
};

use axum::{
    Form, Json, Router, ServiceExt,
    body::{Body, Bytes},
    extract::{FromRequest, Request, State, rejection::FormRejection},
    http::{HeaderName, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, get_service, post},
};
use event_listener::Event;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;
use toasty::{Db, driver};
use tokio::{select, signal};
use tower::ServiceBuilder;
// use tower_cookies::CookieManagerLayer;
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::{error, info_span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use validator::{Validate, ValidationError, ValidationErrors};

fn assert_sync_send<T: Send>(_: T) {}

#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(length(min = 1, message = "username should not be null"))]
    username: String,
}

async fn root() -> &'static str {
    "hello world!"
}

async fn create_user(
    state: State<AppState>,
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // let user = User {
    //     id: 1,
    //     username: payload.username,
    // };

    println!("==>find_by_id(u1.id)");
    let user = db::User::find_by_email("zhangsan@qq.com")
        .get(&state.db)
        .await
        .unwrap();
    println!("==>find_by_id(u1.id) = {user:#?}");

    let user = User {
        id: 1,
        username: user.name,
    };

    (StatusCode::CREATED, Json(user))
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404")
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Received termination signal shutting down");

    // let handle = axum_server::Handle::new();

    // let shutdown_future = shutdown_signal(handle.clone());

    // tokio::spawn(redirect_http_to_https(ports, shutdown_future));

    // handle.graceful_shutdown(Some(Duration::from_secs(10)));
}

const REQUEST_ID_HEADER: &str = "x-req-id";

#[derive(Clone)]
struct AppState {
    db: Arc<toasty::Db>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any)
        .max_age(Duration::from_secs(60) * 10);

    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                // Log the request id as generated.
                let request_id = request.headers().get(REQUEST_ID_HEADER);

                match request_id {
                    Some(request_id) => info_span!(
                        "http_request",
                        request_id = ?request_id,
                    ),
                    None => {
                        error!("could not extract request_id");
                        info_span!("http_request")
                    }
                }
            }),
        )
        .layer(PropagateRequestIdLayer::new(x_request_id));

    let schema_file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schema.toasty");

    println!("{:#?}", &schema_file);

    let schema = toasty::schema::from_file(schema_file)?;

    // // println!("{:#?}", &schema);

    let db_file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("toasty.db");

    let driver = toasty_sqlite::Sqlite::open(db_file)?;

    let db = Arc::new(toasty::Db::new(schema, driver).await);

    let state = AppState { db: db.clone() };

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .nest_service(
            "/static",
            get_service(ServeDir::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
            )))
            .handle_error(|error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {error}"),
                )
            }),
        )
        .fallback(handler_404)
        // .layer(CookieManagerLayer::new())
        .layer(cors)
        .layer(middleware)
        .layer(middleware::from_fn(print_request_response))
        .with_state(state);

    // db.reset_db().await?;

    // assert_sync_send(db::User::find_by_email("hello").first(&db));

    // println!("==>u1=User::create()");
    // let u1 = db::User::create()
    //     .name("zhangsan")
    //     .email("zhangsan@qq.com")
    //     .exec(&db)
    //     .await?;

    // println!("==>u2=User::create()");
    // let u2 = db::User::create()
    //     .name("lisi")
    //     .email("lisi@qq.com")
    //     .exec(&db)
    //     .await?;

    // println!("==>find_by_id(u1.id)");
    // let user = db::User::find_by_email("zhangsan@qq.com").get(&db).await?;
    // println!("==>find_by_id(u1.id) = {user:#?}");

    // println!("==>find_by_email(u1.id)");
    // let mut user = db::User::find_by_email(&u1.email).get(&db).await?;
    // println!("==>find_by_email(u1.id) = {user:#?}");

    // user.update().name("wangwu").exec(&db).await?;

    // let todo = u2.todos().create().title("title").exec(&db).await?;
    // println!("CREATED = {todo:#?}");

    // let mut todos = u2.todos().all(&db).await?;
    // while let Some(todo) = todos.next().await {
    //     let todo = todo.unwrap();
    //     println!("TODO = {todo:#?}");
    //     println!("-> user {:?}", todo.user().find(&db).await.unwrap());
    // }

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
}

async fn print_request_response(
    req: axum::extract::Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = axum::extract::Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{direction} body = {body:?}");
    }

    Ok(bytes)
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    ValidationError(#[from] ValidationErrors),

    #[error(transparent)]
    AxumFormRejection(#[from] FormRejection),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::ValidationError(_) => {
                let message = format!("Input validation error: [{self}]").replace('\n', ", ");
                (StatusCode::BAD_REQUEST, message)
            }
            ServerError::AxumFormRejection(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        }
        .into_response()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedInput<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedInput<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = ServerError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedInput(value))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        thread,
        time::Duration,
    };

    use event_listener::{Event, Listener, listener};

    #[test]
    fn test_listener() {
        let flag = Arc::new(AtomicBool::new(false));

        let event = Arc::new(Event::new());

        thread::spawn({
            let flag = flag.clone();
            let event = event.clone();
            move || {
                thread::sleep(Duration::from_secs(1));

                flag.store(true, Ordering::SeqCst);

                event.notify(usize::MAX);
            }
        });

        loop {
            println!("------------");
            if flag.load(Ordering::SeqCst) {
                println!("-----1-------");
                break;
            }

            listener!(event => listener);

            // let listener = event.listen();
            if flag.load(Ordering::SeqCst) {
                println!("-----2-------");
                break;
            }
            listener.wait();
        }
    }
}
