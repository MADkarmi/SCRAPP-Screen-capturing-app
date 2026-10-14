# Scrap Desktop App

## System Preparation
Note that FFmpeg v8.0 is required.
### Windows
1. Install vcpkg https://github.com/microsoft/vcpkg?tab=readme-ov-file#quick-start-windows
2. Install FFmpeg ```vcpkg install ffmpeg --triplet x64-windows```
3. Set ```VCPKG_ROOT``` to where ```vcpkg``` is installed
4. Add ```%VCPKG_ROOT%\installed\x64-windows\bin``` to your path
5. Run ```vcpkg install llvm[clang,target-x86]:x64-windows```
6. Set ```LIBCLANG_PATH``` to where clang is installed: ```%VCPKG_ROOT%\installed\x64-windows\bin```
### Linux
On Linux, you need to install ffmpeg and alsa. For example on Debian based systems:
```
sudo apt-get install clang libavcodec-dev libavformat-dev libavutil-dev libavfilter-dev libavdevice-dev libasound2-dev pkg-config
```
### macOS
```
brew install pkg-config ffmpeg
```

in simple terms:
Install LLVM (through official installer, Visual Studio, Chocolatey, or any other means), and add LLVM's bin path to PATH, or set LIBCLANG_PATH to that (see clang-sys documentation for additional info).
Install FFmpeg (complete with headers) through any means, e.g. downloading a pre-built "full_build-shared" version from https://ffmpeg.org/download.html. Set FFMPEG_DIR to the directory containing include and lib.


## About
Simple app for screen capturing
## ToDo:
- [x] Multichoice listview of video and audio sources

- [x] Settings Tab

- [x] File path picker

- [x] Mic & System Audio capture (cpal)

- [x] List devices

- [x] Create thread for enabled input

- [x] Video capture (Scap)

- [x] encode audio and video (ffmpeg) ```cargo add ffmpeg-next``` https://stackoverflow.com/questions/75935019/using-rust-to-execute-ffmpeg-commands https://www.reddit.com/r/rust/comments/15qm23z/rust_and_ffmpeg/ https://subvisual.com/blog/posts/real-time-video-processing-with-rust-ffmpeg-opencv/ https://vijiboy.wordpress.com/2018/03/08/ffmpeg-command-line-examples-for-audio-video-capture/

- [X] fix memory leak when rescaling frames

- [x] implement conditional device selection

- [x] prevent from starting recording without selected video source

- [x] prevent from deactivating video source without selecting another while recording is in session

- [x] handle closing while recorder is running

- [x] implement video source change

- [ ] fix video and audio alignment issues -
Mostly done. Right now we made a tmp fix, so that if sys audio doesn't send any samples then we push silence, and if any samples come, we buffer them.
the proper fix would be to leave audio_thread to run on it's own timeline. If no samples come, then we write silence. If samples appear, then we take what we require (#channels * 1024) and resample them and push to mixer. If to few samples appear, then we write silence to the rest of frame. And we control on the next audio_thread clock tick, if samples which are in ring buffer from audio recorder are from past (should be in previous frame, then we drop them) and we continue as usual.

- [ ] Currently captured frame preview

- [x] Save user settings

- [x] Add multiple audio devices handling
    1. [x] mixing
    2. [x] resampling

- [ ] fix memory leak when running from .exe

- [x] Implement sending captured video to site api

### To Do 4 Cpal
- [x] Add support for all available hosts (not only default)
- [x] Add support for all possible output & input formats


- [ ] implement SIMD-accelerated mixing of audio samples

## Setup and Usage

1. Install Rust by following the [Rust Getting Started Guide](https://www.rust-lang.org/learn/get-started).
   Once this is done, you should have the ```rustc``` compiler and the ```cargo``` build system installed in your path.

2. Change fields in .env
    ```
        OAUTH_AUTHORIZATION_ENDPOINT
        OAUTH_TOKEN_ENDPOINT
        OAUTH_REDIRECT_URI
        OAUTH_LOGOUT_ENDPOINT
        OAUTH_POST_LOGOUT_REDIRECT_URI
        OAUTH_REVOCATION_ENDPOINT
        UPLOAD_API_ENDPOINT
    ```
    to the uris set in web app
3. Build with cargo
    ```
    cargo build
    ```
4. Run the application binary
     ```
     cargo run
     ```
5. If you want to enable experimental features run
    ```
    cargo run --features experimental
    ```
