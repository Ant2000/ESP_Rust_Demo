pub mod nvs;
pub mod storage;
pub mod littlefs;

// let val = get_u8(0x0).await;
// info!("Value: {}", val);
//
// store_u8(0x0, val + 1).await;
//
// let filesystem = fs.lock().await;
// let path = path!("/littlefs/test.txt");
//
//
// match filesystem.open_file_with_options_and_then(
//     |options| { options.read(true) },
//     path,
//     |file| {
//         let mut buf = [0u8; 128];
//         file.read(&mut buf).map(
//             |bytes_read| {
//                 let s = core::str::from_utf8(&buf[..bytes_read]).unwrap();
//                 info!("Read: {}", s);
//             }
//         )
//     }
// ) {
//     Ok(_) => {}
//     Err(e) => {
//         error!("Error: {:?}", e);
//     }
// };
//
// filesystem.create_dir(path!("/littlefs")).ok();
//
// match filesystem.open_file_with_options_and_then(
//     |options| {options.write(true).truncate(true).create(true)},
//     path,
//     |file| {
//         let mut buf: String<128> = String::new();
//         write!(buf, r#"{{"num":{}}}"#, val).unwrap();
//         file.write_all(buf.as_bytes())
//     }
// ) {
//     Ok(_) => {}
//     Err(e) => {
//         error!("Error: {:?}", e);
//     }
// }