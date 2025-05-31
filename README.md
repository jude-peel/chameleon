# Chameleon

A no-dependency Rust image manipulation utility (eventually).

## Status

The chunk parsing, decompression, and filtering all work well. Right now, the
PNG decoder can successfully convert simple pictures using the RGB color type.
The next steps are to tackle interlacing, the other color types, and then add
support for as many optional ancillary chunks as possible (and worth doing).

Thanks to PngSuite by Willem van Schaik, I now have a good way to test and fix
all of the many edge cases the PNG format provides.

## Goals

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
  - [ ] Color types.
    - [ ] Grayscale.
    - [x] RGB.
    - [ ] Palette index.
    - [ ] Grayscale + alpha.
    - [ ] RGB + alpha.
  - [ ] Interlacing.
    - [x] None.
    - [ ] Adam7 (AAAAAAAAAAA).
  - [ ] Ancillary chunks.
    - [ ] tRNS
    - [ ] gAMA
    - [ ] cHRM
    - [ ] sRGB
    - [ ] iCCP
    - [ ] tEXt
    - [ ] zTXt
    - [ ] iTXt
    - [ ] bKGD
    - [ ] pHYs
    - [ ] sBIT
    - [ ] sPLT
    - [ ] hIST
    - [ ] tIME
