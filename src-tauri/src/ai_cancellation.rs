use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

#[derive(Clone)]
pub struct ActiveCancellationRequest {
    session_uuid: String,
    request_id: String,
    receiver: watch::Receiver<bool>,
}

impl ActiveCancellationRequest {
    pub fn is_cancelled(&self) -> bool {
        *self.receiver.borrow()
    }

    pub async fn cancelled(&mut self) {
        if *self.receiver.borrow() {
            return;
        }

        while self.receiver.changed().await.is_ok() {
            if *self.receiver.borrow() {
                return;
            }
        }
    }
}

impl Drop for ActiveCancellationRequest {
    fn drop(&mut self) {
        AI_CANCELLATION_MANAGER.finish_request(&self.session_uuid, &self.request_id);
    }
}

pub struct AICancellationManager {
    active_requests: Arc<Mutex<HashMap<String, (String, watch::Sender<bool>)>>>,
}

impl AICancellationManager {
    pub fn new() -> Self {
        Self {
            active_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn begin_request(&self, session_uuid: &str) -> Result<ActiveCancellationRequest, String> {
        let mut active_requests = self
            .active_requests
            .lock()
            .map_err(|error| format!("锁定 AI 取消管理器失败: {error}"))?;

        let request_id = uuid::Uuid::new_v4().to_string();
        let (sender, receiver) = watch::channel(false);

        if let Some((_, previous_sender)) =
            active_requests.insert(session_uuid.to_string(), (request_id.clone(), sender))
        {
            let _ = previous_sender.send(true);
        }

        Ok(ActiveCancellationRequest {
            session_uuid: session_uuid.to_string(),
            request_id,
            receiver,
        })
    }

    pub fn cancel_request(&self, session_uuid: &str) -> Result<bool, String> {
        let active_requests = self
            .active_requests
            .lock()
            .map_err(|error| format!("锁定 AI 取消管理器失败: {error}"))?;

        let Some((_, sender)) = active_requests.get(session_uuid) else {
            return Ok(false);
        };

        sender.send(true).map(|_| true).or_else(|_| Ok(true))
    }

    pub fn finish_request(&self, session_uuid: &str, request_id: &str) {
        let Ok(mut active_requests) = self.active_requests.lock() else {
            return;
        };

        let should_remove = active_requests
            .get(session_uuid)
            .map(|(active_request_id, _)| active_request_id == request_id)
            .unwrap_or(false);

        if should_remove {
            active_requests.remove(session_uuid);
        }
    }
}

lazy_static::lazy_static! {
    pub static ref AI_CANCELLATION_MANAGER: AICancellationManager = AICancellationManager::new();
}
