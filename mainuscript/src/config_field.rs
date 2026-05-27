use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Mutex, MutexGuard};
pub struct ConfigField<T: Sized + Sync + Debug>{
    inner: Mutex<T>,
    subscribers: HashMap<usize, Box<dyn Fn(&T) + Sync + Send>>,
}

impl<T: Sized + Sync + Debug> ConfigField<T>{
    pub fn new(value: T) -> ConfigField<T>{
        ConfigField { inner: Mutex::new(value), subscribers: HashMap::new() }
    }
    pub fn get_value(&self) -> MutexGuard<'_, T>{
        // let _lock = self.inner.lock().unwrap();
        return self.inner.lock().expect("get_value in ConfigField");
    }
    pub fn subscribe(&mut self, callback: Box<dyn Fn(&T) + Send + Sync>) -> usize{
        //Get new free index
        let mut current_len = self.subscribers.len();
        while self.subscribers.contains_key(&current_len) {
            current_len = current_len.wrapping_add(1);
        }

        self.subscribers.insert(current_len, callback);
        return current_len;
    }
    pub fn clear_subscription(&mut self, id: usize){
        self.subscribers.remove(&id);
    }
    pub fn ignorant_update(&mut self, value: T) -> T{
        let mut lock = self.inner.lock().expect("ignorant_update in ConfigField");
        let old_value: T = std::mem::replace(&mut *lock, value);
        return old_value;
    }
    pub fn update(&mut self, value: T) -> T{
        let mut lock = self.inner.lock().expect("update in ConfigField");
        for (_, callback) in self.subscribers.iter(){
            callback(&value)
        };
        let old_value: T = std::mem::replace(&mut *lock, value);
        return old_value;
    }
}
impl<T: Sized + Sync + Debug> Debug for ConfigField<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Inner: {:?}, subscribers: {}", self.inner, self.subscribers.len()))
    }
}
pub struct ConfigSubscription<'parent_config, T: Sized + Sync + Debug>{
    id: usize,
    parent_config: &'parent_config mut ConfigField<T>
}
impl<'parent_config, T: Sized + Sync + Debug> ConfigSubscription<'parent_config, T>{
    pub fn update(&mut self, value: T) -> T{
        let mut lock = self.parent_config.inner.lock().expect("update in ConfigSubscription");
        for (id, callback) in self.parent_config.subscribers.iter(){
            if *id != self.id {
                callback(&value) //Should'nt this be an error?
            }
        };
        let old_value: T = std::mem::replace(&mut *lock, value);
        return old_value;
    }
}
impl<'parent_config, T: Sized + Sync + Debug> Drop for ConfigSubscription<'parent_config, T>{
    fn drop(&mut self) {
        self.parent_config.clear_subscription(self.id);
    }
}