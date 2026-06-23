use esp_hal::peripherals::FLASH;
use static_cell::StaticCell;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embedded_storage::nor_flash::{NorFlash as BlockingNorFlash, ReadNorFlash as BlockingReadNorFlash};
use embedded_storage_async::nor_flash::{NorFlash, ReadNorFlash, ErrorType};
use esp_storage::FlashStorage;
use crate::storage::nvs::init_nvs;

static SHARED_FLASH: StaticCell<Mutex<CriticalSectionRawMutex, FlashStorage<'static>>> = StaticCell::new();

#[derive(Copy, Clone)]
pub struct SharedFlash {
    pub flash: &'static Mutex<CriticalSectionRawMutex, FlashStorage<'static>>,
    pub capacity: usize,
}

impl ErrorType for SharedFlash {
    type Error = <FlashStorage<'static> as ErrorType>::Error;
}

impl ReadNorFlash for SharedFlash {
    const READ_SIZE: usize = <FlashStorage<'static> as BlockingReadNorFlash>::READ_SIZE;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.flash.lock().await.read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        self.capacity
    }
}

impl NorFlash for SharedFlash {
    const WRITE_SIZE: usize = <FlashStorage<'static> as BlockingNorFlash>::WRITE_SIZE;

    const ERASE_SIZE: usize = <FlashStorage<'static> as BlockingNorFlash>::ERASE_SIZE;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.flash.lock().await.erase(from, to)
    }
    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.flash.lock().await.write(offset, bytes)
    }
}

pub fn initialise_flash(esp_flash: FLASH<'static>) {
    let flash = FlashStorage::new(esp_flash);
    let capacity = flash.capacity();
    let mutex = SHARED_FLASH.init(Mutex::new(flash));
    let shared_flash = SharedFlash {flash: mutex, capacity };
    init_nvs(shared_flash);
}