# Chameleon
A no-dependency Rust image manipulation utility (eventually).
## Goals
- [ ] CLI
    - [x] Get file input path and output path.
    - [ ] Write usage and help blurb.
    - [ ] Expand with feature set.
- [ ] PNG Decoder
    - [x] Read PNG file header.
    - [x] Get all chunks.
    - [x] Check the CRC on each chunk.
    - [ ] Decompress IDAT.
        - [x] Concatenate ZLIB bitstreams from IDAT chunks.
        - [x] Parse ZLIB header/adler32.
        - [ ] DEFLATE decompression. 
            - [x] Block type 0.
            - [x] Block type 1.
                - [x] Prefix code decoding.
                - [x] LZSS decoding.
            - [ ] Block type 2.
                - [ ] Dynamic prefix code tree generation.
                - [ ] Same things from block type 1 but slightly different.
    - [ ] Filters.
      - [x] None.
      - [x] Sub.
      - [x] Up.
      - [ ] Average.
      - [ ] Paeth.
    - [ ] Learn more.
