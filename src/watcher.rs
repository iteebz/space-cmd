use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SessionEvent {
    pub provider: String,
    pub session_id: String,
    pub line: String,
}

pub struct FileWatcher {
    tx: mpsc::Sender<SessionEvent>,
    rx: mpsc::Receiver<SessionEvent>,
    sessions_dir: PathBuf,
    tracked_files: HashMap<PathBuf, u64>,
}

impl FileWatcher {
    pub fn new(sessions_dir: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel();
        FileWatcher {
            tx,
            rx,
            sessions_dir,
            tracked_files: HashMap::new(),
        }
    }

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.poll_sessions()?;
        Ok(())
    }

    fn poll_sessions(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.sessions_dir.exists() {
            return Ok(());
        }

        for provider_dir in fs::read_dir(&self.sessions_dir)? {
            let provider_path = provider_dir?.path();
            if !provider_path.is_dir() {
                continue;
            }

            let provider = provider_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            for session_file in fs::read_dir(&provider_path)? {
                let session_path = session_file?.path();
                if session_path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
                    continue;
                }

                if let Ok(metadata) = fs::metadata(&session_path) {
                    let size = metadata.len();
                    let last_size = self.tracked_files.get(&session_path).copied();

                    if last_size.is_some() && Some(size) != last_size {
                        self.read_new_lines(&session_path, last_size.unwrap(), &provider)?;
                    }

                    self.tracked_files.insert(session_path, size);
                }
            }
        }

        Ok(())
    }

    fn read_new_lines(
        &self,
        path: &PathBuf,
        start_pos: u64,
        provider: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let start_idx = start_pos as usize;
        let new_content = if start_idx < content.len() {
            &content[start_idx..]
        } else {
            ""
        };

        let session_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        for line in new_content.lines() {
            if !line.is_empty() {
                let event = SessionEvent {
                    provider: provider.to_string(),
                    session_id: session_id.clone(),
                    line: line.to_string(),
                };
                let _ = self.tx.send(event);
            }
        }

        Ok(())
    }

    pub fn poll(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.poll_sessions()
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Option<SessionEvent> {
        self.rx.recv_timeout(timeout).ok()
    }

    pub fn try_recv(&self) -> Option<SessionEvent> {
        self.rx.try_recv().ok()
    }
}
