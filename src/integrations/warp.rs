use warp::http::Response;
use warp::hyper::StatusCode;
use warp::http::header;

use crate::Template;

/// Return a response with the rendered template contents
/// Sets the Content-Type header to text/html
pub fn reply_template<T: Template>(t: &T) -> warp::reply::Response {
    // TODO: maybe make mime types available at runtime so this can be better
    match t.render() {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html")
            .body(content.into()),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Error Rendering Template this is likely a bug in stilts".into())
    }.unwrap()
}
