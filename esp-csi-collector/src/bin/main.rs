#![no_std]
#![no_main]

extern crate alloc;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use esp_bootloader_esp_idf::esp_app_desc;
use esp_hal::{
    clock::CpuClock,
    rng::Rng,
    timer::timg::TimerGroup,
};
use esp_println::println;

use esp_wifi::{
    init as wifi_init,
    wifi::{WifiController, Interfaces},
    EspWifiController,
};

use esp_csi_rs::{
    collector::CSISniffer,
    config::CSIConfig,
};

esp_app_desc!();

macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        STATIC.uninit().write($val)
    }};
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // ----- System Setup -----
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 128 * 1024);

    // Embassy 初期化
    let tg1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(tg1.timer0);

    println!("=== ESP32S3 CSI Sniffer (USB serial -> Raspberry Pi) ===");

    // ----- WiFi init -----
    let tg0 = TimerGroup::new(peripherals.TIMG0);
    let timer0 = tg0.timer0;
    let rng = Rng::new(peripherals.RNG);

    // esp-wifi の初期化
    let init = wifi_init(timer0, rng).unwrap();
    // StaticCell に載せて 'static にする
    let init_ref: &'static mut EspWifiController<'static> =
        mk_static!(EspWifiController<'static>, init);

    let wifi = peripherals.WIFI;

    // WiFi controller + interfaces を取得
    let (wifi_ctrl, interfaces): (WifiController<'static>, Interfaces<'static>) =
        esp_wifi::wifi::new(init_ref, wifi).unwrap();

    // ----- CSISniffer の構築 -----
    let mut sniffer = CSISniffer::new(CSIConfig::default(), wifi_ctrl).await;

    // CSI sniffer の初期化（タスクなどを内部で張る）
    sniffer
        .init(interfaces, &spawner)
        .await
        .expect("CSISniffer init failed");

    // CSI collection start
    sniffer.start_collection().await;
    println!("CSI Sniffer started. Printing CSI to USB serial...");

    // ---- メインループ ----
    loop {
        // ライブラリの組み込みヘルパで CSI + メタデータを丸ごと print
        // （mac, rssi, channel, csi raw data 配列など）
        sniffer.print_csi_w_metadata().await;

        // ほんの少しだけ息継ぎ（なくてもいいが、他タスク配慮で 1ms）
        Timer::after(Duration::from_millis(1)).await;
    }
}
