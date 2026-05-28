use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
struct QrEntry {
    url: String,
    updated_at_secs: u64,
}

#[derive(Clone, Default)]
pub struct QrUrlStore {
    urls: Arc<Mutex<HashMap<String, QrEntry>>>,
}

impl QrUrlStore {
    pub fn new() -> Self {
        Self {
            urls: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set(&self, session_id: &str, url: &str) {
        if let Ok(mut guard) = self.urls.lock() {
            guard.insert(
                session_id.to_string(),
                QrEntry {
                    url: url.to_string(),
                    updated_at_secs: now_secs(),
                },
            );
        }
    }

    pub fn touch(&self, session_id: &str) {
        if let Ok(mut guard) = self.urls.lock() {
            if let Some(entry) = guard.get_mut(session_id) {
                entry.updated_at_secs = now_secs();
            }
        }
    }

    pub fn get(&self, session_id: &str, expire_secs: u64) -> Option<String> {
        let mut guard = self.urls.lock().ok()?;
        let entry = guard.get(session_id)?;
        if expire_secs > 0 && now_secs().saturating_sub(entry.updated_at_secs) > expire_secs {
            guard.remove(session_id);
            return None;
        }
        Some(entry.url.clone())
    }

    pub fn has_fresh(&self, session_id: &str, expire_secs: u64) -> bool {
        self.get(session_id, expire_secs).is_some()
    }

    pub fn remove(&self, session_id: &str) {
        if let Ok(mut guard) = self.urls.lock() {
            guard.remove(session_id);
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
