use chrono::{DateTime, Duration, Utc};

pub struct DebounceActor<T> {
    duration: Duration,
    sender: tokio::sync::mpsc::Sender<T>,
    last_sent_data: Option<LastSentData>,
}

impl<T: Send + Sync + 'static> DebounceActor<T> {
    pub fn new(duration: Duration, sender: tokio::sync::mpsc::Sender<T>) -> Self {
        Self {
            last_sent_data: None,
            duration,
            sender,
        }
    }

    pub async fn handle_new_value(&mut self, msg: T) {
        let now = Utc::now();
        let sender = self.sender.clone();

        let duration = match &self.last_sent_data {
            Some(last_sent_data) => last_sent_data.next_duration(now, self.duration),
            None => None,
        };

        let task = tokio::task::spawn(async move {
            if let Some(Ok(duration)) = duration.map(|x| x.to_std()) {
                tokio::time::sleep(duration).await;
            }
            let _ = sender.send(msg).await;
        });

        self.last_sent_data = Some(LastSentData::new(now, duration, task));
    }
}

struct LastSentData {
    last_sent: DateTime<Utc>,
    previous_duration: Option<Duration>,
    task: tokio::task::JoinHandle<()>,
}

impl LastSentData {
    fn new(
        last_sent: DateTime<Utc>,
        previous_duration: Option<Duration>,
        task: tokio::task::JoinHandle<()>,
    ) -> Self {
        Self {
            last_sent,
            previous_duration,
            task,
        }
    }

    fn next_duration(&self, now: DateTime<Utc>, max_duration: Duration) -> Option<Duration> {
        let value = match (self.task.is_finished(), self.previous_duration) {
            (true, None) => {
                let since_last_send = now - self.last_sent;
                if since_last_send > max_duration {
                    None
                } else {
                    Some(max_duration)
                }
            }
            (true, Some(_)) => {
                let since_last_send = now - self.last_sent;
                Some(max_duration.max(since_last_send - max_duration))
            }
            (false, None) => None,
            (false, Some(_)) => {
                let since_last_send = now - self.last_sent;
                Some(max_duration.max(since_last_send - max_duration))
            }
        };
        self.task.abort();
        value
    }
}
