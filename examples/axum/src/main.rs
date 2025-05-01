use axum::{
	extract::{FromRequest, Request},
	http::StatusCode,
	response::{IntoResponse, Response},
	routing::{get, post},
	Json, Router,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use wary::Wary;

#[tokio::main]
async fn main() {
	// build our application with a route
	let app = Router::new()
		// `GET /` goes to `root`
		.route("/", get(root))
		// `POST /users` goes to `create_user`
		.route("/users", post(create_user));

	// run our app with hyper, listening globally on port 3000
	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

struct Valid<T>(pub T);

#[derive(Debug, Error)]
enum ValidRejection {
	#[error("Invalid JSON")]
	Json(#[from] axum::extract::rejection::JsonRejection),
	#[error("Invalid payload")]
	Wary(#[from] wary::Report),
}

impl IntoResponse for ValidRejection {
	fn into_response(self) -> Response {
		match self {
			Self::Json(err) => err.into_response(),
			Self::Wary(err) => {
				let mut response = Json(err).into_response();
				*response.status_mut() = StatusCode::BAD_REQUEST;
				response
			}
		}
	}
}

impl<S, T> FromRequest<S> for Valid<T>
where
	T: DeserializeOwned + Wary<S>,
	S: Send + Sync,
{
	type Rejection = ValidRejection;

	async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
		let mut content: Json<T> = Json::from_request(req, state).await?;
		content.0.wary(state)?;

		Ok(Valid(content.0))
	}
}

// basic handler that responds with a static string
async fn root() -> &'static str {
	"Hello, World!"
}

#[axum::debug_handler]
async fn create_user(
	// this argument tells axum to parse the request body
	// as JSON into a `CreateUser` type
	Valid(payload): Valid<CreateUser>,
) -> (StatusCode, Json<User>) {
	// insert your application logic here
	let user = User {
		id: 1337,
		username: payload.username,
	};

	// this will be converted into a JSON response
	// with a status code of `201 Created`
	(StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize, Wary)]
struct CreateUser {
	#[validate(ascii, length(3..=16))]
	#[transform(lowercase(ascii))]
	username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
	id: u64,
	username: String,
}
