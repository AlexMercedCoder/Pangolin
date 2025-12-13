use std::env;

pub struct EnvGuard {
    key: String,
    original_value: Option<String>,
}

impl EnvGuard {
    pub fn new(key: &str, value: &str) -> Self {
        let original_value = env::var(key).ok();
        env::set_var(key, value);
        Self {
            key: key.to_string(),
            original_value,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.original_value {
            Some(v) => env::set_var(&self.key, v),
            None => env::remove_var(&self.key),
        }
    }
}
