pub fn clear_screen() {
    if cfg!(target_os = "windows") {
        println!("{}", "\n".repeat(30));
    } else {
        print!("\x1b[2J\x1b[1;1H");
    }
}

pub struct OutputBuffer {
    buffer: String,
}

impl OutputBuffer {
    pub fn with_capacity(capacity: usize) -> OutputBuffer {
        OutputBuffer {
            buffer: String::with_capacity(capacity),
        }
    }

    pub fn append(&mut self, s: &str) {
        self.buffer.push_str(&s);
    }

    pub fn flush(&mut self) {
        print!("{}", &self.buffer);
        self.buffer.clear();
    }

    pub fn clear_screen(&mut self) {
        if cfg!(target_os = "windows") {
            self.append(&"\n".repeat(50));
        } else {
            self.append("\x1b[2J\x1b[1;1H");
        }
    }
}