use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct QrUrlStore {
    urls: Arc<Mutex<HashMap<String, String>>>,
}

impl QrUrlStore {
    pub fn new() -> Self {
        Self {
            urls: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set(&self, session_id: &str, url: &str) {
        if let Ok(mut guard) = self.urls.lock() {
            guard.insert(session_id.to_string(), url.to_string());
        }
    }

    pub fn get(&self, session_id: &str) -> Option<String> {
        self.urls.lock().ok()?.get(session_id).cloned()
    }

    pub fn remove(&self, session_id: &str) {
        if let Ok(mut guard) = self.urls.lock() {
            guard.remove(session_id);
        }
    }
}
