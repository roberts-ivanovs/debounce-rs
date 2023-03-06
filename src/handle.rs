use chrono::Duration;

use crate::actor::DebounceActor;

#[derive(Debug)]
pub struct Debounce<T> {
    receiver: tokio::sync::mpsc::Receiver<T>,
    sender: tokio::sync::mpsc::Sender<T>,
    #[allow(dead_code)]
    task: tokio::task::JoinHandle<()>,
}

const DEBOUNCE_VALUE_BUFFER: usize = 1;
const DEBOUNCE_RESPONSE_BUFFER: usize = 1;

impl<T: Send + Sync + 'static> Debounce<T> {
    pub fn new(duration: Duration) -> Self {
        let (sender_val, receiver_val) = tokio::sync::mpsc::channel(DEBOUNCE_VALUE_BUFFER);
        let (sender_resp, receiver_resp) = tokio::sync::mpsc::channel(DEBOUNCE_RESPONSE_BUFFER);
        let actor = DebounceActor::new(duration, sender_resp);

        let task = tokio::spawn(async move {
            run(actor, receiver_val).await;
        });

        Self {
            receiver: receiver_resp,
            sender: sender_val,
            task,
        }
    }

    pub async fn set(&self, value: T) {
        let _ = self.sender.send(value).await;
    }

    pub async fn get(&mut self) -> T {
        self.receiver.recv().await.expect("Debounce actor died")
    }
}

async fn run<T: Send + Sync + 'static>(
    mut actor: DebounceActor<T>,
    mut receiver: tokio::sync::mpsc::Receiver<T>,
) {
    while let Some(value) = receiver.recv().await {
        actor.handle_new_value(value).await;
    }
}
