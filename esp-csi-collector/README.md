# ESP32S3 CSI Collector

A WiFi Channel State Information (CSI) collector for ESP32S3 that operates in **Sniffer mode**. The ESP32S3 monitors all nearby WiFi traffic and outputs CSI data via USB serial port to a Raspberry Pi or host computer.

## Features

- **CSI Collection**: Captures Channel State Information from WiFi packets
- **Sniffer Mode**: Monitors all nearby WiFi traffic without requiring connection
- **USB Serial Output**: CSI data and metadata output via USB serial port
- **No Configuration**: No WiFi credentials or network settings needed
- **Simple Setup**: Minimal codebase, easy to understand and modify
- **High Performance**: Uses Embassy async framework for efficient operation

## Architecture

```
┌─────────────┐         USB Serial          ┌─────────────┐
│  ESP32S3    │─────────────────────────────►│ Raspberry Pi│
│  (Sniffer)  │                               │             │
│             │                               │ 1. Read     │
│ 1. Monitor  │                               │    Serial   │
│    WiFi     │                               │             │
│    Traffic  │                               │ 2. Parse    │
│             │                               │    CSI Data │
│ 2. Collect  │                               │             │
│    CSI      │                               │ 3. Save to  │
│             │                               │    File     │
│ 3. Output   │                               │             │
│    via USB  │                               └─────────────┘
└─────────────┘
```

**How it works:**
1. ESP32S3 operates in sniffer mode, monitoring all WiFi traffic
2. When CSI data is detected, it's collected along with metadata
3. CSI data is printed to USB serial using `print_csi_w_metadata()`
4. Raspberry Pi reads serial data and saves it to a file
5. CSI data can be parsed and analyzed offline

## Hardware Requirements

- **ESP32S3** development board (with USB-JTAG-SERIAL support)
- **Raspberry Pi** or any computer with USB port and Python
- **USB Cable** to connect ESP32S3 to Raspberry Pi

**Note**: No WiFi router connection needed! Sniffer mode works independently.

## Software Requirements

- Rust with `xtensa` toolchain
- `espflash` for flashing the ESP32S3
- `cargo` and `rustup`
- Python 3 with `pyserial` library (for Raspberry Pi)

### Installing ESP Rust Toolchain

```bash
# Install espup
cargo install espup

# Install ESP Rust toolchain
espup install

# Source the environment (add to your shell profile)
source ~/export-esp.sh  # or export-esp.ps1 on Windows
```

### Installing espflash

```bash
cargo install espflash
```

### Installing Python Dependencies (Raspberry Pi)

```bash
pip3 install pyserial
```

## Configuration

**No configuration needed!** Sniffer mode doesn't require WiFi credentials or network settings.

The `.cargo/config.toml.example` file contains only build configuration. You can copy it if you want to customize build settings:

```bash
cd esp-csi-collector
cp .cargo/config.toml.example .cargo/config.toml
# Edit if needed (usually not necessary)
```

## Building and Flashing

### Build the Project

```bash
cd esp-csi-collector
cargo build --release
```

### Flash to ESP32S3

```bash
cargo run --release
```

This will automatically build, flash, and open a serial monitor.

### Monitor Serial Output

If you want to just monitor after flashing:

```bash
espflash monitor
```

## Raspberry Pi Setup

### Receiving CSI Data via Serial

Connect ESP32S3 to Raspberry Pi via USB cable, then run the provided Python script:

```bash
python3 csi_receiver.py
```

The script will:
1. Open serial port `/dev/tty.usbmodem101` (adjust if needed)
2. Read CSI data line by line
3. Save to `csi_log.txt` file

### Customizing Serial Port

Edit `csi_receiver.py` to change the serial port:

```python
# For Linux (usually /dev/ttyACM0 or /dev/ttyUSB0)
ser = serial.Serial("/dev/ttyACM0", 115200)

# For macOS (usually /dev/tty.usbmodem*)
ser = serial.Serial("/dev/tty.usbmodem101", 115200)

# For Windows (usually COM3, COM4, etc.)
ser = serial.Serial("COM3", 115200)
```

### Parsing CSI Data

The CSI data is output in a human-readable format. Example output:

```
mac: D6:62:A7:DC:DF:7C
rssi: -79
rate: 9
sig_mode: 0
mcs: 0
channel: 1
timestamp: 26123538
csi raw data:
[0, 0, 0, 0, -12, 9, -13, 8, ...]
```

You can parse this data using Python:

```python
import re

def parse_csi_line(line):
    """Parse a single line of CSI output."""
    if line.startswith("mac:"):
        return {"type": "mac", "value": line.split(":")[1].strip()}
    elif line.startswith("rssi:"):
        return {"type": "rssi", "value": int(line.split(":")[1].strip())}
    elif line.startswith("csi raw data:"):
        return {"type": "csi_start"}
    # ... add more parsers as needed
```

## CSI Data Format

The CSI data is output via `print_csi_w_metadata()` method, which formats the `CSIDataPacket` structure. The output includes:

- **MAC Address**: Source MAC address of the WiFi packet
- **RSSI**: Received Signal Strength Indicator (dBm)
- **Rate**: PHY rate encoding
- **Channel**: WiFi channel number
- **Timestamp**: Local timestamp (microseconds)
- **CSI Raw Data**: Array of CSI measurements (typically 128 bytes)
- **Other Metadata**: Signal mode, MCS, bandwidth, antenna, etc.

Refer to the [esp-csi-rs documentation](https://docs.rs/esp-csi-rs/latest/esp_csi_rs/struct.CSIDataPacket.html) for detailed field descriptions.

## How It Works

### Initialization Flow

1. **System Setup**: Initialize HAL, Embassy runtime, heap allocator
2. **WiFi Initialization**: Create WiFi controller and interfaces
3. **CSISniffer Creation**: Create sniffer with default configuration
4. **Initialization**: Call `init()` to start internal tasks
5. **Start Collection**: Call `start_collection()` to enable CSI collection
6. **Main Loop**: Continuously call `print_csi_w_metadata()` to output CSI data

### Internal Tasks (Handled by Library)

The `esp-csi-rs` library spawns internal tasks that handle:
- WiFi sniffer mode configuration
- CSI hardware enable/disable
- Packet capture and CSI extraction
- Data formatting for output

## Troubleshooting

### Build Errors

If you see compilation errors:

```bash
# Clean and rebuild
cargo clean
cargo build --release
```

### Flash Errors

If flashing fails:

```bash
# Try holding the BOOT button on ESP32S3 while flashing
cargo run --release

# Or specify the port explicitly
espflash flash --port /dev/ttyACM0 target/xtensa-esp32s3-none-elf/release/esp-csi-collector
```

### No CSI Data

If you're not seeing CSI data:

1. Check that there is WiFi traffic in the area
2. Verify the ESP32S3 antenna connection
3. Check serial output for error messages
4. Ensure sniffer is started (look for "CSI Sniffer started" message)

### Serial Port Issues

If Raspberry Pi can't read serial data:

1. Check USB cable connection
2. Verify serial port name (use `ls /dev/tty*` on Linux/Mac)
3. Check permissions: `sudo chmod 666 /dev/ttyACM0`
4. Ensure baud rate matches (115200)
5. Try different USB ports

### Serial Monitor Issues

If the serial monitor is garbled:

```bash
# Try different baud rates
espflash monitor --speed 115200
```

## Sniffer Mode vs Station Mode

This implementation uses **Sniffer Mode**, which is simpler and requires no network configuration.

**Sniffer Mode** (current implementation):
- ✅ No WiFi credentials needed
- ✅ Monitors all nearby WiFi traffic
- ✅ Simple setup
- ✅ USB serial output
- ❌ Cannot send UDP responses (no network connection)

**Station Mode** (alternative):
- Requires WiFi credentials
- Connects to WiFi router
- Can send UDP responses
- More complex setup
- Requires network stack

## Data Collection Tips

1. **Channel Selection**: The sniffer monitors the channel configured in `CSIConfig`. Check the esp-csi-rs documentation for channel configuration options.

2. **Data Rate**: CSI data is output continuously. Adjust the delay in the main loop if you need to control the output rate.

3. **File Management**: The Python script appends to `csi_log.txt`. Consider rotating logs or using timestamps in filenames for long-term collection.

4. **Parsing**: The output format is text-based, making it easy to parse with standard text processing tools (grep, awk, Python, etc.).

## References

- [esp-csi-rs GitHub Repository](https://github.com/Connected-Motion-Research/esp-csi-rs)
- [esp-csi-rs Documentation](https://docs.rs/esp-csi-rs/latest/esp_csi_rs/)
- [ESP-IDF CSI Documentation](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/wifi.html#wi-fi-channel-state-information)
- [ESP Rust Book](https://esp-rs.github.io/book/)

## License

This project inherits the license from its dependencies. Please refer to individual crate licenses for details.

## Support

For issues specific to this implementation, please check:
- ESP32S3 datasheet
- esp-csi-rs documentation
- esp-hal and esp-wifi documentation

---

**Built with 🦀 Rust for ESP32S3**
