use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TransferResult {
    pub source: String,
    pub dest_host: String,
    pub dest_path: String,
    pub bytes: u64,
    pub duration_ms: u64,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_path: Option<String>,
}

impl TransferResult {
    pub fn print_human(&self) {
        println!("sent: {}", self.source);
        println!("to:   {}:{}", self.dest_host, self.dest_path);
        if self.bytes > 0 {
            println!("size: {}", format_bytes(self.bytes));
        }
        if self.duration_ms > 0 {
            println!("time: {}ms", self.duration_ms);
        }
        println!("ok");
    }

    pub fn print_json(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            println!("{}", json);
        }
    }
}

pub enum Output {
    Human,
    Json,
}

impl Output {
    pub fn print(&self, result: &TransferResult) {
        match self {
            Output::Human => result.print_human(),
            Output::Json => result.print_json(),
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
