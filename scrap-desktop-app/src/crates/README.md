## Why the `ffmpeg-next` source code is included

The project includes the full source code of the `ffmpeg-next` crate because a small modification had to be made to its internal `Frame::data_mut()` implementation in

The original implementation used the `linesize` value corresponding to the requested plane:

```rust
(*self.as_ptr()).linesize[index] as usize
```
However, for audio frames, FFmpeg does not populate the linesize array in the same way as it does for video frames. In particular, the audio frame can have only the first linesize element populated with a non-zero value, while the remaining elements are zero.

As a result, using linesize[index] for audio planes could produce an incorrect slice length (including a zero-length slice for planes other than the first one).

To work around this behavior, the data_mut() implementation was modified to always use linesize[0]:

``` rust
pub fn data_mut(&mut self, index: usize) -> &mut [u8] {
    if index >= self.planes() {
        panic!("out of bounds");
    }

    unsafe {
        slice::from_raw_parts_mut(
            (*self.as_mut_ptr()).data[index],
            (*self.as_ptr()).linesize[0] as usize,
        )
    }
}
```

Why the entire source was included

ffmpeg-next is an external dependency and normally its source is obtained automatically by Cargo from crates.io. Since the project requires a modification to the crate itself, the unmodified dependency cannot be used directly.

The complete ffmpeg-next source is therefore included in the repository so that Cargo can build the project using the locally modified version. This also makes the modification reproducible: another developer can clone the repository and build the project without having to manually patch the dependency after downloading it.

This modification is intentionally kept as small as possible and only changes the way the mutable audio-frame buffer length is obtained. The rest of the ffmpeg-next implementation remains unchanged.

The actual modification is limited to
```
ffmpeg-next/
└── src/
    └── util/
        └── frame/
            └── audio.rs   <-- modified
```
