# Chameleon

A no-dependency Rust image manipulation utility (eventually).

## Goals

Currently, the PNG decoder works, however only on small images and the
breaking point needs to be found. Some problems seem to come from the zlib
decompresser.

- [ ] CLI
  - [x] Get file input path and output path.
  - [ ] Write usage and help blurb.
  - [ ] Expand with feature set.
- [ ] PNG Decoder
  - [x] Read PNG file header.
  - [x] Get all chunks.
  - [x] Check the CRC on each chunk.
  - [x] Decompress IDAT.
    - [x] Concatenate ZLIB bitstreams from IDAT chunks.
    - [x] Parse ZLIB header/adler32.
    - [x] DEFLATE decompression.
      - [x] Block type 0.
      - [x] Block type 1.
        - [x] Prefix code decoding.
        - [x] LZSS decoding.
      - [x] Block type 2.
        - [x] Dynamic prefix code tree generation.
        - [x] Same things from block type 1 but slightly different.
  - [x] Filters.
    - [x] None.
    - [x] Sub.
    - [x] Up.
    - [x] Average.
    - [x] Paeth.
  - [ ] Learn more.
