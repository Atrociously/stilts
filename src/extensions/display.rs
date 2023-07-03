use std::fmt::Display;

/// An extension for types that implement [`Display`]
pub trait DisplayExt {
    /// Create a formatter that will escape
    /// the the display output so that it is safe to
    /// put into html contexts
    fn escape(&self) -> EscapeHtml<'_, Self>;

    /// Convert the display output to lowercase
    fn lowercase(&self) -> String;

    /// Convert the display output to uppercase
    fn uppercase(&self) -> String;
}

impl<T> DisplayExt for T
where
    T: Display,
{
    fn escape(&self) -> EscapeHtml<'_, Self> {
        EscapeHtml(self)
    }

    fn lowercase(&self) -> String {
        self.to_string().to_lowercase()
    }

    fn uppercase(&self) -> String {
        self.to_string().to_uppercase()
    }
}

pub struct EscapeHtml<'a, T: ?Sized>(&'a T);

impl<T> Display for EscapeHtml<'_, T>
where
    T: ?Sized + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = self.0.to_string();
        let escaped = html_escape::encode_safe(&content);
        Display::fmt(&escaped, f)
    }
}
