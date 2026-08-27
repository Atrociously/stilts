use std::path::Path;

pub trait FileSystem {
    fn read_to_string<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<String>;
}

pub struct StdFS;

impl FileSystem for StdFS {
    fn read_to_string<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }
}
