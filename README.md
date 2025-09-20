# Ziee Chat

## Development

### React Front End

1. Clone the repo
2. Install Node.js (v18 or higher): https://nodejs.org/en/download
3. Install dependencies: `npm install`
4. Build the front end: `npm run build`
5. Start the front end server: `npm run dev`

### Rust Back End
1. Install Rust: https://www.rust-lang.org/tools/install
2. Install Tauri Prerequisites: https://v2.tauri.app/start/prerequisites/
3. Install llvm, clang, and cmake
   - On Ubuntu: `sudo apt install llvm clang cmake`
   - On MacOS: `brew install llvm cmake`
   - On Windows: Install via Visual Studio Installer
     - llvm/clang: https://github.com/llvm/llvm-project/releases/tag/llvmorg-18.1.8
     - make
     - cmake: https://cmake.org/download/
     - cuda-sdk
     - cudnn
     - vulkan-sdk
     - winflexbison3
     - winlibs
     - cygwin
     - strawberryperl
     - check `dev-utils/windows-dev-env-activate.bat` for activation of the environment
4. Build the back end: `cargo build` or `cargo build --release`
5. Start the back end server: 
   - Run as web server: `APP_DATA_DIR=path/to/store/app/data HEADLESS=true cargo run --bin ziee`
   - Run as desktop app: `APP_DATA_DIR=path/to/store/app/data cargo run --bin ziee`
   
   - `APP_DATA_DIR` is the directory where the app data will be stored. It can be any directory you choose.
   By default, on Mac and Linux, it will use `~/.ziee` and on Windows, it will use `%APPDATA%/ziee`.
6. The app will be available at `http://localhost:1430` for web server mode or as a desktop app.