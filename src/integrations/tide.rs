use crate::Template;

/// Return a response with the rendered template contents
/// Sets the Content-Type header to text/html
pub fn render_template<T: Template>(t: &T) -> tide::Response {
    match t.render() {
        Ok(content) => {
            let mut response = tide::Response::builder(tide::StatusCode::Ok);
            if let Some(mime) = t.mime_str() {
                response = response.header("Content-Type", mime);
            }
            response.body(content)
                .build()
        }
        Err(_) => tide::Response::builder(tide::StatusCode::InternalServerError)
            .body("Error Rendering Template this is likely a bug in stilts")
            .build(),
    }
}
