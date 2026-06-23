use embassy_sync::blocking_mutex::{raw::CriticalSectionRawMutex};
use embassy_sync::mutex::Mutex;
use embassy_sync::once_lock::OnceLock;
use sequential_storage::cache::NoCache;
use sequential_storage::map::{MapConfig, MapStorage};
use serde::{de::DeserializeOwned, Serialize};
use crate::storage::storage::SharedFlash;

type Storage = MapStorage<u8, SharedFlash, NoCache>;
type StorageType = Mutex<CriticalSectionRawMutex, Storage>;
type StorageBuffer = Mutex<CriticalSectionRawMutex, [u8; 256]>;
static STORAGE: OnceLock<StorageType> = OnceLock::new();
static SCRATCH_BUFFER: OnceLock<StorageBuffer> = OnceLock::new();


pub fn init_nvs(esp_flash: SharedFlash) {
    let storage = MapStorage::<u8, _, _>::new(esp_flash, const {MapConfig::new(0x9000..0xE000)}, NoCache::new());

    match STORAGE.init(Mutex::new(storage)) {
        Ok(_) => (),
        Err(_) => panic!("Failed to init NVS"),
    };

    match SCRATCH_BUFFER.init(Mutex::new([0; 256])) {
        Ok(_) => (),
        Err(_) => panic!("Failed to init scratch buffer"),
    };
}

pub async fn store_u8(key: u8, value: u8) {
    let mut storage = STORAGE.get().await.lock().await;
    let mut buffer = SCRATCH_BUFFER.get().await.lock().await;

    storage.store_item(&mut *buffer, &key, &value).await.unwrap();
}

pub async fn get_u8(key: u8) -> u8 {
    let mut storage = STORAGE.get().await.lock().await;
    let mut buffer = SCRATCH_BUFFER.get().await.lock().await;

    storage
        .fetch_item::<u8>(&mut *buffer, &key)
        .await
        .unwrap()
        .unwrap_or(0)
}

pub async fn store_u16(key: u8, value: u16) {
    let mut storage = STORAGE.get().await.lock().await;
    let mut buffer = SCRATCH_BUFFER.get().await.lock().await;

    storage.store_item(&mut *buffer, &key, &value).await.unwrap();
}

pub async fn get_u16(key: u8) -> u16 {
    let mut storage = STORAGE.get().await.lock().await;
    let mut buffer = SCRATCH_BUFFER.get().await.lock().await;

    storage
        .fetch_item::<u16>(&mut *buffer, &key)
        .await
        .unwrap()
        .unwrap_or(0)
}

pub async fn store_u32(key: u8, value: u32) {
    let mut storage = STORAGE.get().await.lock().await;
    let mut buffer = SCRATCH_BUFFER.get().await.lock().await;

    storage.store_item(&mut *buffer, &key, &value).await.unwrap();
}

pub async fn get_u32(key: u8) -> u32 {
    let mut storage = STORAGE.get().await.lock().await;
    let mut buffer = SCRATCH_BUFFER.get().await.lock().await;

    storage
        .fetch_item::<u32>(&mut *buffer, &key)
        .await
        .unwrap()
        .unwrap_or(0)
}

pub async fn store_u64(key: u8, value: u64) {
    let mut storage = STORAGE.get().await.lock().await;
    let mut buffer = SCRATCH_BUFFER.get().await.lock().await;

    storage.store_item(&mut *buffer, &key, &value).await.unwrap();
}

pub async fn get_u64(key: u8) -> u64 {
    let mut storage = STORAGE.get().await.lock().await;
    let mut buffer = SCRATCH_BUFFER.get().await.lock().await;

    storage
        .fetch_item::<u64>(&mut *buffer, &key)
        .await
        .unwrap()
        .unwrap_or(0)
}

pub async fn store_f32(key: u8, value: f32) {
    let mut storage = STORAGE.get().await.lock().await;
    let mut buffer = SCRATCH_BUFFER.get().await.lock().await;

    storage.store_item(&mut *buffer, &key, &value).await.unwrap();
}

pub async fn get_f32(key: u8) -> f32 {
    let mut storage = STORAGE.get().await.lock().await;
    let mut buffer = SCRATCH_BUFFER.get().await.lock().await;

    storage
        .fetch_item::<f32>(&mut *buffer, &key)
        .await
        .unwrap()
        .unwrap_or(0.0)
}

pub async fn store_f64(key: u8, value: f64) {
    let mut storage = STORAGE.get().await.lock().await;
    let mut buffer = SCRATCH_BUFFER.get().await.lock().await;

    storage.store_item(&mut *buffer, &key, &value).await.unwrap();
}

pub async fn get_f64(key: u8) -> f64 {
    let mut storage = STORAGE.get().await.lock().await;
    let mut buffer = SCRATCH_BUFFER.get().await.lock().await;

    storage
        .fetch_item::<f64>(&mut *buffer, &key)
        .await
        .unwrap()
        .unwrap_or(0.0)
}

pub async fn store_blob(key: u8, value: &[u8]) {
    let mut storage = STORAGE.get().await.lock().await;
    let mut scratch = SCRATCH_BUFFER.get().await.lock().await;

    storage
        .store_item(&mut *scratch, &key, &value)
        .await
        .unwrap();
}

pub async fn get_blob(
    key: u8,
    out: &mut [u8],
) -> usize {
    let mut storage = STORAGE.get().await.lock().await;
    let mut scratch = SCRATCH_BUFFER.get().await.lock().await;

    if let Some(blob) = storage
        .fetch_item::<&[u8]>(&mut *scratch, &key)
        .await
        .unwrap()
    {
        let len = core::cmp::min(blob.len(), out.len());
        out[..len].copy_from_slice(&blob[..len]);
        len
    } else {
        0
    }
}
