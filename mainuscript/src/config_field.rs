use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};
use borsh::{BorshSerialize, BorshDeserialize};

// #[derive(BorshSerialize, BorshDeserialize)]
#[derive(Default)]
pub struct ConfigField<T: Sized + Send + Sync + Debug + Clone>{
    // #[borsh(skip)]
    inner: RwLock<T>,
    subscribers: RwLock<HashMap<usize, Box<dyn Fn(&T) + Send + Sync>>>,
}
impl<T: Sized + Send + Sync + Debug + Clone + BorshSerialize + BorshDeserialize> BorshSerialize for ConfigField<T> {
    fn serialize<W: std::io::prelude::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let inner = self.inner.read().unwrap();
        T::serialize(&inner, writer)
    }
}
impl<T: Sized + Send + Sync + Debug + Clone + BorshSerialize + BorshDeserialize> BorshDeserialize for ConfigField<T>{
    fn deserialize_reader<R: std::io::prelude::Read>(reader: &mut R) -> std::io::Result<Self> {
        let deserialized_inner = T::deserialize_reader(reader)?;
        return Ok(ConfigField::new(deserialized_inner));
    }
}

impl<T: Sized + Send + Sync + Debug + Clone> ConfigField<T>{
    pub fn new(value: T) -> ConfigField<T>{
        ConfigField { inner: RwLock::new(value), subscribers: RwLock::new(HashMap::new()) }
    }
    pub fn get_value_rw_lock(&self) -> &RwLock<T>{
        // let _lock = self.inner.lock().unwrap();
        return &self.inner;
    }
    pub fn into_inner(self) -> T{
        return self.inner.into_inner().unwrap();
    }
    fn subscribe(&self, callback: Box<dyn Fn(&T) + Send + Sync>) -> usize{
        //Get new free index
        let mut subscribers_ref = self.subscribers.write().unwrap();
        let mut current_len = subscribers_ref.len();
        while subscribers_ref.contains_key(&current_len) {
            current_len = current_len.wrapping_add(1);
        }

        subscribers_ref.insert(current_len, callback);
        return current_len;
    }
    fn clear_subscription(&self, id: usize){
        self.subscribers.write().unwrap().remove(&id);
    }
    pub fn ignorant_update(&self, value: T) -> T{
        let mut lock = self.inner.write().expect("ignorant_update in ConfigField");
        let old_value: T = std::mem::replace(&mut *lock, value);
        return old_value;
    }
    pub fn update(&self, value: T) -> T{
        let mut lock = self.inner.write().expect("update in ConfigField");

        let old_value: T = std::mem::replace(&mut *lock, value.clone());
        drop(lock);
        for (_, callback) in self.subscribers.read().unwrap().iter(){
            callback(&value)
        };
        return old_value;
    }
}
impl<T: Sized + Send + Sync + Debug + Clone> Debug for ConfigField<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Inner: {:?}, subscribers: {}", self.inner, self.subscribers.read().unwrap().len()))
    }
}
#[derive(Debug)]
pub struct ConfigSubscription<T: Sized + Send + Sync + Debug + Clone>{
    id: usize,
    parent_config: Arc<ConfigField<T>>
}
impl<T: Sized + Send + Sync + Debug + Clone> ConfigSubscription<T>{
    pub fn subscribe(config: Arc<ConfigField<T>>, callback:Box<dyn Fn(&T) + Send + Sync> ) -> ConfigSubscription<T>{
        return ConfigSubscription { id: config.subscribe(callback), parent_config: config.clone() }
    }
    pub fn update(&mut self, value: T) -> T{
        self.parent_config.update(value)

        // let mut lock = self.parent_config.write().unwrap();
        // for (id, callback) in lock.subscribers.iter(){
        //     if *id != self.id {
        //         callback(&value) //Should'nt this be an error?
        //     }
        // };
        // let old_value: T = std::mem::replace(&mut *lock, value);
        // return old_value;
    }
}
impl<T: Sized + Send + Sync + Debug + Clone> Drop for ConfigSubscription<T>{
    fn drop(&mut self) {
        self.parent_config.clear_subscription(self.id);
    }
}