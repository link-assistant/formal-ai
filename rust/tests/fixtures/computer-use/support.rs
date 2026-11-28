use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};

pub const CORPUS: &str = include_str!("tasks.lino");

pub struct Publisher {
    child: Child,
    pub origin: String,
}

impl Publisher {
    pub fn start() -> Self {
        let script = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/computer-use/serve.py"
        );
        let child = Command::new("python3")
            .arg(script)
            .stdout(Stdio::piped())
            .spawn()
            .expect("launch fixture publisher");
        let mut publisher = Self {
            child,
            origin: String::new(),
        };
        let stream = publisher.child.stdout.take().expect("publisher stdout");
        BufReader::new(stream)
            .read_line(&mut publisher.origin)
            .expect("read fixture publisher origin");
        publisher.origin = publisher.origin.trim_end().to_owned();
        assert!(publisher.origin.starts_with("http://127.0.0.1:"));
        publisher
    }

    pub fn corpus(&self) -> String {
        CORPUS.replace("fixture://", &format!("{}/", self.origin))
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
