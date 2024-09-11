pub struct Buffer {
    pub lines: Vec<String>,
}

impl Default for Buffer {
    fn default() -> Self {
        let mut lines = Vec::new();
        lines.push(String::from("Hello world"));
        Self { lines }
    }
}

impl Buffer {
    pub fn len(&self) -> usize {
        self.lines.len()
    }
}
