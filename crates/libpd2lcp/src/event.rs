use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};

use crate::error::Error;

#[derive(Clone, Debug)]
pub enum Event {
    // This is a catch-all for any error it might encounter.
    None,
    Error(String),
    DownloadingWine,
    FinishedDownloadingWine,
    InitPrefix,
    FinishedInitPrefix,
    UpdatingPD2 { done: u32, total: u32 },
}

#[derive(Clone, Default, Debug)]
pub struct EventNotify {
    inner: Arc<(Mutex<VecDeque<Event>>, Condvar)>,
}

unsafe impl Send for EventNotify {}
unsafe impl Sync for EventNotify {}

impl EventNotify {
    pub fn wait_event(&self) -> Result<Vec<Event>, Error> {
        let (guard, condvar) = &*self.inner;

        let mut lock = condvar.wait_while(guard.lock()?, |i| i.is_empty())?;

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
