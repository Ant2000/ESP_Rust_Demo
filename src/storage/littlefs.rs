use embedded_storage::Storage;
use embedded_storage::nor_flash::{NorFlash as BlockingNorFlash, ReadNorFlash as BlockingReadNorFlash};
use embedded_storage::ReadStorage;
use esp_storage::{FlashStorage};
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use littlefs2::fs::{Filesystem, Allocation};
use littlefs2::driver::Storage as FsStorage;
use esp_bootloader_esp_idf::partitions::{read_partition_table, DataPartitionSubType, FlashRegion, PartitionType, PARTITION_TABLE_MAX_LEN};
use log::error;
use static_cell::StaticCell;
use crate::storage::storage::SharedFlash;
use crate::mk_static;

pub type FsType = Mutex<CriticalSectionRawMutex, Filesystem<'static, LittleFsStorage>> ;
static FILESYSTEM: StaticCell<FsType> = StaticCell::new();


pub struct LittleFsStorage {
    flash: FlashRegion<'static, SharedFlash>,
}

impl FsStorage for LittleFsStorage {
    const READ_SIZE: usize = <FlashStorage as BlockingReadNorFlash>::READ_SIZE;
    const WRITE_SIZE: usize = <FlashStorage as BlockingNorFlash>::WRITE_SIZE;
    const BLOCK_SIZE: usize = <FlashStorage as BlockingNorFlash>::ERASE_SIZE;
    const BLOCK_COUNT: usize = 0x430000 / Self::BLOCK_SIZE;
    const BLOCK_CYCLES: isize = 100;

    type CACHE_SIZE = littlefs2::consts::U256;
    type LOOKAHEAD_SIZE = littlefs2::consts::U128;

    fn read(&mut self, off: usize, buf: &mut [u8]) -> littlefs2::io::Result<usize> {
        match ReadStorage::read(&mut self.flash, off as u32, buf) {
            Ok(_) => {
                Ok(buf.len())
            }
            Err(_) => {
                Err(littlefs2::io::Error::IO)
            }
        }
    }

    fn write(&mut self, off: usize, data: &[u8]) -> littlefs2::io::Result<usize> {
        match Storage::write(&mut self.flash, off as u32, data) {
            Ok(_) => {
                Ok(data.len())
            }
            Err(_) => {
                Err(littlefs2::io::Error::IO)
            }
        }
    }

    fn erase(&mut self, off: usize, len: usize) -> littlefs2::io::Result<usize> {
        match self.flash.erase(off as u32, (off + len) as u32) {
            Ok(_) => {
                Ok(len)
            }
            Err(_) => {
                Err(littlefs2::io::Error::IO)
            }
        }
    }
}

pub fn init_filesystem(esp_flash: &'static mut SharedFlash) -> &'static mut FsType {
    let pt_buf = mk_static!([u8; PARTITION_TABLE_MAX_LEN], [0u8; PARTITION_TABLE_MAX_LEN]);

    let table = read_partition_table(esp_flash, pt_buf).expect("Failed to read partition table");
    let entry = table.find_partition(PartitionType::Data(DataPartitionSubType::LittleFs))
        .expect("Flash read error")
        .expect("LittleFS partition is not active");

    let region = entry.as_embedded_storage(esp_flash);
    let storage = mk_static!(LittleFsStorage, LittleFsStorage { flash: region });

    let alloc= mk_static!(Allocation<LittleFsStorage>, Allocation::new());

    let needs_format = Filesystem::mount(alloc, storage).is_err();

    if needs_format {
        error!("Mount failed, formatting...");
        Filesystem::format(storage).expect("format failed");
    }

    let filesystem = Filesystem::mount(alloc, storage)
        .expect("Failed to mount LittleFS");
    FILESYSTEM.init(Mutex::new(filesystem))
}