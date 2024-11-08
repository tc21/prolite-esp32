things you need to configure if you want to build this on your own:
- the build target (in `.cargo/config.toml` and `rust-toolchain.toml` in each project) should point to your chip model; I have an esp32s3
- `.cargo/config.toml` in each project: `--flash-size 16mb` should be updated according to your esp32 (you'll need 2mb minimum)
- `controller/.cargo/config.toml`: fill in WIFI_SSID and WIFI_PASSWORD
- `controller/src/main.rs`: configure your uart pins
- `driver/src/config.rs` and `driver/src/main.rs`: configure your uart and control (output) pins
- look at `glyphs*.txt` and the `.py` files in `driver/`: you may want to add, update, or generate your own glyphs; then run the two generate_glyphs.py scripts


hardware requirements:
- 2 esp32 boards, one of them with wifi capabilities
- a pro-lite m2014r display
- power supply for the above things
- the ability to solder or connect wires between your esp32 and the pro-lite display
- understanding of the control (input) pin layout of your prolite display; see part 1.1  of (my writeup)[https://natsuai.com/personal/writeup-20241016/index.html] for details


why 2 boards?

turns out a wifi server is too much for a puny esp32 to handle without taking a way too much core time from the driver thread. so we need to offload it to another esp32 so the display is crisp and without artifacts. the renderer also affects display quality but I figured it was better than sending 20-60 frame buffers over serial each second
