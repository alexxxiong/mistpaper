//! One pending snapshot; serialization and file replacement run on a worker.
use crate::model::Shelf;
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Condvar, Mutex, OnceLock},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const DEBOUNCE: Duration = Duration::from_millis(150);
const MAX_DELAY: Duration = Duration::from_secs(1);
static STORE: OnceLock<Store> = OnceLock::new();

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum SaveStatus {
    #[default]
    Saved,
    Saving,
    Failed,
}

struct Pending {
    snapshot: Option<Shelf>,
    changed: Instant,
    first: Instant,
    closing: bool,
    revision: u64,
    status: SaveStatus,
}

pub(crate) struct Store {
    initial: Shelf,
    shared: Arc<(Mutex<Pending>, Condvar)>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl Store {
    pub(crate) fn open(dir: &Path) -> io::Result<Self> {
        let path = dir.join("library.json");
        let initial = match fs::read(&path) {
            Ok(bytes) => {
                let shelf: Shelf = serde_json::from_slice(&bytes)?;
                shelf.validate().map_err(io::Error::other)?;
                shelf
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Shelf::default(),
            Err(e) => return Err(e),
        };
        let shared = Arc::new((
            Mutex::new(Pending {
                snapshot: None,
                changed: Instant::now(),
                first: Instant::now(),
                closing: false,
                revision: 0,
                status: SaveStatus::Saved,
            }),
            Condvar::new(),
        ));
        let state = shared.clone();
        let worker = thread::Builder::new()
            .name("mistpaper-save".into())
            .spawn(move || write_loop(path, state))?;
        Ok(Self {
            initial,
            shared,
            worker: Mutex::new(Some(worker)),
        })
    }

    pub(crate) fn initial(&self) -> Shelf {
        self.initial.clone()
    }

    pub(crate) fn submit(&self, shelf: Shelf) {
        let mut pending = self.shared.0.lock().unwrap();
        if pending.snapshot.is_none() {
            pending.first = Instant::now();
        }
        pending.snapshot = Some(shelf);
        pending.changed = Instant::now();
        pending.revision += 1;
        pending.status = SaveStatus::Saving;
        self.shared.1.notify_one();
    }

    pub(crate) fn status(&self) -> SaveStatus {
        self.shared.0.lock().unwrap().status
    }

    pub(crate) fn close(&self) {
        {
            let mut pending = self.shared.0.lock().unwrap();
            pending.closing = true;
            self.shared.1.notify_one();
        }
        if let Some(worker) = self.worker.lock().unwrap().take() {
            worker.join().expect("save worker");
        }
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        self.close();
    }
}

fn write_loop(path: PathBuf, shared: Arc<(Mutex<Pending>, Condvar)>) {
    loop {
        let (shelf, revision) = {
            let mut p = shared.0.lock().unwrap();
            loop {
                if p.snapshot.is_some() {
                    let remaining = DEBOUNCE
                        .saturating_sub(p.changed.elapsed())
                        .min(MAX_DELAY.saturating_sub(p.first.elapsed()));
                    if p.closing || remaining.is_zero() {
                        break;
                    }
                    p = shared.1.wait_timeout(p, remaining).unwrap().0;
                } else if p.closing {
                    return;
                } else {
                    p = shared.1.wait(p).unwrap();
                }
            }
            (p.snapshot.take().unwrap(), p.revision)
        };
        let result = write_snapshot(&path, &shelf);
        if let Err(ref error) = result {
            log::error!("saving library: {error}");
        }
        let mut p = shared.0.lock().unwrap();
        if p.revision == revision {
            p.status = if result.is_ok() {
                SaveStatus::Saved
            } else {
                SaveStatus::Failed
            };
        }
    }
}

fn write_snapshot(path: &Path, shelf: &Shelf) -> io::Result<()> {
    fs::create_dir_all(path.parent().expect("library directory"))?;
    let temp = path.with_extension("json.tmp");
    let file = fs::File::create(&temp)?;
    let mut writer = io::BufWriter::new(file);
    serde_json::to_writer(&mut writer, shelf)?;
    use io::Write;
    writer.flush()?;
    writer.get_ref().sync_all()?;
    drop(writer);
    fs::rename(temp, path)
}

pub(crate) fn initialize(dir: &Path) -> io::Result<()> {
    STORE
        .set(Store::open(dir)?)
        .map_err(|_| io::Error::other("store already initialized"))
}

pub(crate) fn store() -> &'static Store {
    STORE.get().expect("store initialized before UI")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_flushes_latest_snapshot_and_reopens_it() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        let mut shelf = store.initial();
        for i in 0..100 {
            shelf.docs[0].body = format!("最新中文草稿{i}").into();
            store.submit(shelf.clone());
        }
        store.close();
        let reopened = Store::open(dir.path()).unwrap();
        assert_eq!(&*reopened.initial().docs[0].body, "最新中文草稿99");
        assert!(!dir.path().join("library.json.tmp").exists());
    }

    #[test]
    fn malformed_existing_library_is_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("library.json");
        fs::write(&path, b"incomplete library").unwrap();
        assert!(Store::open(dir.path()).is_err());
        assert_eq!(fs::read(path).unwrap(), b"incomplete library");
    }

    #[test]
    fn snapshots_share_large_bodies() {
        let mut shelf = Shelf::default();
        shelf.docs[0].body = "中文长稿".repeat(75000).into();
        let snapshot = shelf.clone();
        assert!(Arc::ptr_eq(&shelf.docs[0].body, &snapshot.docs[0].body));
    }
}
