use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use tokio::sync::Notify;

use crate::error::Error;

#[derive(Clone, Debug)]
pub enum Event {
    // This is a catch-all for any error it might encounter.
    Error(String),
    DownloadingWine,
    FinishedDownloadingWine,
    InitPrefix,
    FinishedInitPrefix,
    UpdatingPD2 { done: u32, total: u32 },
    DoneUpdating,
}

#[derive(Clone, Default, Debug)]
pub struct EventNotify {
    inner: Arc<(Mutex<VecDeque<Event>>, Notify)>,
}

unsafe impl Send for EventNotify {}
unsafe impl Sync for EventNotify {}

impl EventNotify {
    pub async fn wait_event(&self) -> Result<Vec<Event>, Error> {
        let (guard, notify) = &*self.inner;

        notify.notified().await;

        let mut lock = guard.lock()?;

        let mut out = Vec::new();

        while let Some(event) = lock.pop_front() {
            out.push(event);
        }

        Ok(out)
    }

    pub fn notify(&self, event: Event) -> Result<(), Error> {
        let (guard, condvar) = &*self.inner;

        let mut lock = guard.lock()?;

        lock.push_back(event);

        drop(lock);

        condvar.notify_one();

        Ok(())
    }
}

pub async fn wrap_faillable<T, F>(notify: EventNotify, f: F) -> Result<Option<T>, Error>
where
    F: FnOnce() -> Result<T, Error>,
{
    match f() {
        Ok(val) => Ok(Some(val)),
        Err(e) => {
            notify.notify(Event::Error(e.to_string()))?;
            Ok(None)
        }
    }
}
