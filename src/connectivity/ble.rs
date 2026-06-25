use crate::mk_static;
use esp_hal::peripherals::BT;
use bt_hci::controller::ExternalController;
use esp_hal::efuse::{interface_mac_address, InterfaceMacAddress};
use log::{info, warn};
use trouble_host::{peripheral, HostResources};
use esp_radio::ble::controller::BleConnector;
use trouble_host::prelude::*;
use heapless::String;
use core::fmt::Write;
use embassy_futures::join::join;
use static_cell::StaticCell;

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;
/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att

static COUNTER: StaticCell<u8> = StaticCell::new();


#[gatt_service(uuid = "019ef8db-6eb0-7c58-9f46-769675bede23")]
struct GenericService<'a> {
    #[characteristic(uuid = "c6f2f259-b8c4-490e-bffb-6cbbdc3776e1", read, write, notify)]
    sensor_data: heapless::Vec<u8, 244>,
}

#[gatt_server]
struct Server {
    generic: GenericService,
}

pub fn ble_init(bluetooth: BT<'static>, spawner: embassy_executor::Spawner) {
    spawner.spawn(run(bluetooth).unwrap());
}

#[embassy_executor::task]
pub async fn run (bluetooth: BT<'static>) {
    let connector = BleConnector::new(bluetooth, Default::default()).unwrap();
    let controller: ExternalController<_, 1> = ExternalController::new(connector);

    let mac = interface_mac_address(InterfaceMacAddress::Bluetooth);
    info!("Running with mac address {}", mac);

    let mut buffer: String<32> = String::new();
    let bytes = mac.as_bytes();
    let n = bytes.len();
    write!(buffer, "Kazam_{:02X}{:02X}", bytes[n - 2], bytes[n - 1]).unwrap();

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> = HostResources::new();
    let stack = trouble_host::new(controller, &mut resources);
    let Host {mut peripheral, runner, ..} = stack.build();

    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: buffer.as_str(),
        appearance: &appearance::control_device::GENERIC_CONTROL_DEVICE,
    })).unwrap();

    info!("Starting BLE server {}", buffer);

    let _ = join(ble_runner(runner), async {
        let counter_ref = COUNTER.init(0);
        loop {
            match advertise(buffer.as_str(), &mut peripheral, &server).await {
                Ok(conn) => {
                    info!("Connected");
                    if let Err(e) = gatt_events_task(&server, &conn, counter_ref).await {
                        warn!("Error handling GATT events: {:?}", e);
                    }
                    info!("Disconnected");
                }
                Err(e) => {
                    panic!("Error advertising: {:?}", e);
                }
            }
        }
    }).await;
}

async fn ble_runner <C: Controller, P: PacketPool>
(mut runner: Runner<'_, C, P>) {
    loop {
        if let Err(e) = runner.run().await {
            panic!("Error running BLE host: {:?}", e);
        }
    }
}

async fn advertise<'values, 'server, C: Controller>(
    name: &'values str,
    peripheral: &mut Peripheral<'values, C, DefaultPacketPool>,
    server: &'server Server<'values>,
) -> Result<GattConnection<'values, 'server, DefaultPacketPool>, BleHostError<C::Error>> {
    let mut advertiser_data = [0; 31];
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE| BR_EDR_NOT_SUPPORTED),
            AdStructure::CompleteLocalName(name.as_bytes())
        ],
        &mut advertiser_data[..],
    )?;

    let advertiser = peripheral.advertise(
        &Default::default(),
        Advertisement::ConnectableScannableUndirected {
            adv_data: &advertiser_data[..len],
            scan_data: &[]
        }
    ).await?;

    info!("Advertising as {}", name);
    advertiser.accept().await?.with_attribute_server(server).map_err(|e| BleHostError::BleHost(e))
}

async fn gatt_events_task<P:PacketPool> (
    server : &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
    data: &mut u8
) -> Result<(), Error> {
    let sensor_data = &server.generic.sensor_data;

    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected {reason} => break reason,
            GattConnectionEvent::Gatt {event} => {
                match &event {
                    GattEvent::Read(event) => {
                        if(event.handle() == sensor_data.handle) {
                            let mut value = heapless::Vec::<u8, 244>::new();
                            *data += 1;
                            write!(value, "Read Count: {}", *data).unwrap();
                            server.set(sensor_data, &value)?;
                        }
                    }
                    GattEvent::Write(event) => {
                        if(event.handle() == sensor_data.handle) {
                            info!("Write sensor data: {:?}", event.data());
                        }
                    }
                    _ => {}
                };

                match event.accept() {
                    Ok(reply) => {reply.send().await}
                    Err(e) => {warn!("Error sending reply: {:?}", e)}
                };
            }
            _ => {}
        };
    };
    info!("Disconnected with reason {:?}", reason);
    Ok(())
}