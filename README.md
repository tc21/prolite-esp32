things you need to configure if you want to build this on your own:
- the build target (in `.cargo/config.toml` and `rust-toolchain.toml` in each project) should point to your chip model; I have an esp32s3
- `.cargo/config.toml` in each project: `--flash-size 16mb` should be updated according to your esp32 (you'll need 2mb minimum)
- `controller/.cargo/config.toml`: fill in WIFI_SSID and WIFI_PASSWORD
- `controller/src/main.rs`: configure your uart pins
- `driver/src/config.rs` and `driver/src/main.rs`: configure your uart and control (output) pins
- look at `glyphs*.txt` and the `.py` files in `driver/`: you may want to add, update, or generate your own glyphs


hardware requirements:
- 2 esp32 boards, one of them with wifi capabilities
- a pro-lite m2014r display
- power supply for the above things
- the ability to solder or connect wires between your esp32 and the pro-lite display
- understanding of the control (input) pin layout of your prolite display; see part 1.1  of (my writeup)[https://natsuai.com/personal/writeup-20241016/index.html] for details
