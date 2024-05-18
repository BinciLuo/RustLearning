use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use std::sync::atomic::{AtomicBool, Ordering};
use std::fmt;

pub struct Cache {
    cache: Arc<Mutex<HashMap<String, (String, SystemTime)>>>,
    expiration_time: Duration,
    running: Arc<AtomicBool>,
}

impl Cache {
    pub fn new(expiration_time_seconds: u64) -> Self {
        let cache = Arc::new(Mutex::new(HashMap::new()));
        let running = Arc::new(AtomicBool::new(true));
        let expiration_time = Duration::new(expiration_time_seconds, 0);
        let cache_clone = Arc::clone(&cache);
        let running_clone = Arc::clone(&running);

        thread::spawn(move || {
            while running_clone.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_secs(1));
                let now = SystemTime::now();
                let mut cache_lock = cache_clone.lock().unwrap();
                cache_lock.retain(|_, &mut (_, ref expire_time)| {
                    *expire_time > now
                });
            }
        });

        Cache {
            cache,
            expiration_time,
            running,
        }
    }

    pub fn put(&self, key: String, value: String) {
        let expire_time = SystemTime::now() + self.expiration_time;
        let mut cache_lock = self.cache.lock().unwrap();
        cache_lock.insert(key, (value, expire_time));
    }

    pub fn get(&self, key: &str, refresh: bool) -> Option<String> {
        let mut cache_lock = self.cache.lock().unwrap();
        if let Some((val, expire_time)) = cache_lock.get_mut(key) {
            if *expire_time > SystemTime::now() {
                if  refresh {
                    *expire_time = SystemTime::now() + self.expiration_time;
                }
                return Some(val.clone());
            } else {
                cache_lock.remove(key);
            }
        }
        None
    }
    

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn get_expiration_time(&self) -> Duration {
        self.expiration_time
    }

    pub fn get_record_num(&self) -> usize {
        self.cache.lock().unwrap().len()
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

impl fmt::Display for Cache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.is_running() { "Yes" } else { "No" };
        write!(f, "[Cache] \"Running\": {}, \"Expiration Time\": {}s, Record Num: {}.", status, self.get_expiration_time().as_secs(), self.get_record_num())
    }
}

impl fmt::Debug for Cache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
