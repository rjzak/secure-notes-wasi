use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::extract::Path;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Extension, Form, Router};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Default)]
struct State {
    notes: RwLock<HashMap<uuid::Uuid, String>>,
}

#[derive(Deserialize)]
struct NoteForm {
    note: String,
}

const NO_MESSAGE: &str =
    "<html><head><title>Not not found</title></head><body>Note not found</body></html>";

#[cfg_attr(not(target_os = "wasi"), tokio::main)]
#[cfg_attr(target_os = "wasi", tokio::main(flavor = "current_thread"))]
async fn main() -> anyhow::Result<()> {
    let state = State::default();

    #[cfg(not(target_os = "wasi"))]
    {
        use std::net::SocketAddr;
        let addr = SocketAddr::from(([0, 0, 0, 0], 8443));
        axum::Server::bind(&addr)
            .serve(app(state).into_make_service())
            .await?;
    }
    #[cfg(target_os = "wasi")]
    {
        use std::os::wasi::io::FromRawFd;
        tracing::debug!("listening");
        let std_listener = unsafe { std::net::TcpListener::from_raw_fd(3) };
        std_listener
            .set_nonblocking(true)
            .context("failed to set NONBLOCK")?;
        axum::Server::from_tcp(std_listener)
            .context("failed to construct server")?
            .serve(app(state).into_make_service())
            .await?;
    }
    Ok(())
}

fn app(state: State) -> Router {
    Router::new()
        .route("/", get(health))
        .route("/:note-uuid", get(view))
        .route("/new", get(create))
        .route("/new", post(save))
        .layer(Extension(Arc::new(state)))
}

async fn health(Extension(state): Extension<Arc<State>>) -> Html<String> {
    let lock = state.notes.read();
    let num_notes = match lock {
        Ok(l) => l.len(),
        Err(_) => 0,
    };
    Html(format!("<html><head><title>Secure notes has {num_notes}</title><body>We've got {num_notes} notes.<br /><a href=\"new\">Add a note.</a></body></html>"))
}

async fn create() -> Html<String> {
    Html(
        "<!doctype html>
    <html lang=\"en\">
<head><title>New Note</title></head>
<body>
    <form action=\"/new\" method=\"post\">
        <textarea name=\"note\"></textarea> <br />
        <input type=\"submit\" name=\"Save\" />
        <input type=\"reset\" name=\"Reset\" />
    </form>
</body>
</html>"
            .into(),
    )
}

async fn save(
    Extension(state): Extension<Arc<State>>,
    Form(note): Form<NoteForm>,
) -> impl IntoResponse {
    let lock = state.notes.write();
    let next_url = match lock {
        Ok(mut n) => {
            let uuid = Uuid::new_v4();
            n.insert(uuid, note.note);
            format!("/{uuid}")
        }
        Err(_) => "/".into(),
    };

    Redirect::temporary(&next_url)
}

async fn view(
    Extension(state): Extension<Arc<State>>,
    Path(note_uuid): Path<String>,
) -> Html<String> {
    let lock = state.notes.read();
    let uuid = match Uuid::parse_str(&note_uuid) {
        Ok(u) => u,
        Err(_) => return Html(NO_MESSAGE.into()),
    };
    let note = match lock {
        Ok(n) => n.get(&uuid).ok_or("Error").unwrap().clone(),
        Err(_) => return Html(NO_MESSAGE.into()),
    };

    Html(format!(
        "<html><head><title>{uuid}</title><body>{note}</body></html>"
    ))
}
