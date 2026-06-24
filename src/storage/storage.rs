use esp_hal::peripherals::FLASH;
use static_cell::StaticCell;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embedded_storage::nor_flash::{NorFlash as BlockingNorFlash, ReadNorFlash as BlockingReadNorFlash};
use embedded_storage_async::nor_flash::{NorFlash, ReadNorFlash, ErrorType};
use embedded_storage::{Storage, ReadStorage};
use esp_storage::FlashStorage;
use core::cell::RefCell;
use crate::mk_static;
use crate::storage::littlefs::{init_filesystem, FsType};
use crate::storage::nvs::init_nvs;

static SHARED_FLASH: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<FlashStorage<'static>>>> = StaticCell::new();

#[derive(Copy, Clone)]
pub struct SharedFlash {
    pub flash: &'static Mutex<CriticalSectionRawMutex, RefCell<FlashStorage<'static>>>,
    pub capacity: usize,
}

impl ErrorType for SharedFlash {
    type Error = <FlashStorage<'static> as ErrorType>::Error;
}

impl ReadNorFlash for SharedFlash {
    const READ_SIZE: usize = <FlashStorage<'static> as BlockingReadNorFlash>::READ_SIZE;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.flash.lock(|flash| {
            BlockingReadNorFlash::read(&mut *flash.borrow_mut(), offset, bytes)
        })
    }

    fn capacity(&self) -> usize {
        self.capacity
    }
}

impl NorFlash for SharedFlash {
    const WRITE_SIZE: usize = <FlashStorage<'static> as BlockingNorFlash>::WRITE_SIZE;

    const ERASE_SIZE: usize = <FlashStorage<'static> as BlockingNorFlash>::ERASE_SIZE;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.flash.lock(|flash| {
            BlockingNorFlash::erase(&mut *flash.borrow_mut(), from, to)
        })
    }
    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.flash.lock(|flash| {
            BlockingNorFlash::write(&mut *flash.borrow_mut(), offset, bytes)
        })
    }
}

impl BlockingReadNorFlash for SharedFlash {
    const READ_SIZE: usize = <FlashStorage<'static> as BlockingReadNorFlash>::READ_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.flash.lock(|flash| {
            BlockingReadNorFlash::read(&mut *flash.borrow_mut(), offset, bytes)
        })
    }

    fn capacity(&self) -> usize {
        self.capacity
    }

}

impl BlockingNorFlash for SharedFlash {
    const WRITE_SIZE: usize = <FlashStorage<'static> as BlockingNorFlash>::WRITE_SIZE;
    const ERASE_SIZE: usize = <FlashStorage<'static> as BlockingNorFlash>::ERASE_SIZE;
    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.flash.lock(|flash| {
            BlockingNorFlash::erase(&mut *flash.borrow_mut(), from, to)
        })
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.flash.lock(|flash| {
            BlockingNorFlash::write(&mut *flash.borrow_mut(), offset, bytes)
        })
    }
}

impl ReadStorage for SharedFlash {
    type Error = <FlashStorage<'static> as ErrorType>::Error;
    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.flash.lock(|flash| {
            ReadStorage::read(&mut *flash.borrow_mut(), offset, bytes)
        })
    }

    fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Storage for SharedFlash {
    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.flash.lock(|flash| {
            Storage::write(&mut *flash.borrow_mut(), offset, bytes)
        })
    }
}

pub fn initialise_flash(esp_flash: FLASH<'static>) -> &'static mut FsType {
    let flash = FlashStorage::new(esp_flash);
    let capacity = embedded_storage::nor_flash::ReadNorFlash::capacity(&flash);
    let mutex = SHARED_FLASH.init(Mutex::new(RefCell::from(flash)));
    let shared_flash = mk_static!(SharedFlash, SharedFlash {flash: mutex, capacity });
    init_nvs(*shared_flash);
    init_filesystem(&mut *shared_flash)
}