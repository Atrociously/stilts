use crate::Template;

/// Return a response with the rendered template contents
/// Sets the Content-Type header to text/html
pub fn render_template<T: Template>(t: &T) -> tide::Response {
    match t.render() {
        Ok(content) => tide::Response::builder(tide::StatusCode::Ok)
            .header("Content-Type", "text/html")
            .body(content)
            .build(),
        Err(_) => tide::Response::builder(tide::StatusCode::InternalServerError)
            .body("Error Rendering Template this is likely a bug in stilts")
            .build(),
    }
}
