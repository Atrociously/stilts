use std::io;

/// An extension to types that implement [`Serialize`](serde::Serialize)
pub trait SerializeExt {
    /// Create a struct that implements [`Display`] that will
    /// create a json string from the object using its implementation
    /// of [`Serialize`]
    ///
    /// [`Display`]: std::fmt::Display
    /// [`Serialize`]: serde::Serialize
    fn json(&self) -> Jsonify<'_, Self>;
}

impl<T> SerializeExt for T
where
    T: serde::Serialize,
{
    fn json(&self) -> Jsonify<'_, Self> {
        Jsonify(self)
    }
}

pub struct Jsonify<'a, T: ?Sized>(&'a T);

impl<T> std::fmt::Display for Jsonify<'_, T>
where
    T: ?Sized + serde::Serialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_json::to_writer(FmtWriter(f), self.0).map_err(|_| std::fmt::Error)
    }
}

#[cfg_attr(not(feature = "with-json"), allow(dead_code))]
struct FmtWriter<'a, 'b>(&'a mut std::fmt::Formatter<'b>);

impl<'a, 'b> std::io::Write for FmtWriter<'a, 'b> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = String::from_utf8_lossy(buf);
        self.0
            .write_str(&s)
            .map(|_| buf.len())
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "unable to format"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
