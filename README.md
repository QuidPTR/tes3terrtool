A tiny tool to import/export vertex color data into/from TES3 ESM/ESP files. It
is designed to enable easy vertex color editing using external editors.

**Warning**: preliminary, rough and lightly tested.


# Usage

The tool allows to both export and import vertex color data:

* Export lets you specify a source ESM/ESP file and a target RGB 8BPP image
  to which the vertex colors will be dumped.

* Import lets you specify a source ESM/ESP file and a source image holding
  vertex color data (for instance, one you previously exported from the
  source ESM/ESP), and will create a new ESM/ESP that is (ideally) identical
  to the original except the vertex colors will be taken from the image.


# Examples

Exporting:

```sh
tes3terrtool export-vcol --input-esm Morrowind.esm --output-image Morrowind-vcol.bmp
```

Importing:

```sh
tes3terrtool import-vcol --input-esm Morrowind.esm --input-image Morrowind-vcol-edited.bmp --output-esm Morrowind-edited.ems
```

# Installation

Download and unzip the code.  Then run:

```sh
cargo build
```

The executable will be placed in `targets/debug`.  Alternatively, you can run
the tool as, e.g.:

```sh
cargo run -- export-vcol <all other arguments go here>
```


# Caveats

## Data format

ESM/ESP files encode the vertex colors as a 65x65 RGB texture for each cell.
Cells, however, share the vertices on the border, meaning then when created
through regular means (like the Construction Set), the colors of shared
vertices are always identical.

This is why this tool allows exporting/importing colors in two slightly
different ways:

- **tesannwyn dump**: during export, only a 64x64 portion of the texture is
  exported into the target image, whose width and height will be multiples of
  64.  This emulates TESAnnwyn, and it is the default during export.

  *PROs*: no risk of assigning shared vertices two different colors during
  edit. *CONs*: during import, the color of pixels on the bottom and left
  borders of cells with no southern or western neighbor (i.e., the cells at the
  bottom and left borders of the ESM/ESP) are set to white. This is because
  there are no neighboring cells to copy them from.

- **full dump**: during export, the full 65x65 texture is exported into the
  target image, whose width and height will be multiples of 65.  To activate
  this option, use the `-f` flag during export. The import code will
  automatically determine whether the image to be imported is a full dump or
  not.

  *PROs*: full access to the vertex color data. *CONs*: if a shared vertex
  has different colors in the two cells sharing it, the game will override one
  of them. This could potentially result in visible seams in game, so be
  careful when editing the image.

## Image format

The tool exports 8-bit RGB images with no alpha, 24 bits per pixel.

It can import all image formats supported by the Rust `image` module. If the
image contains an alpha channel, this is discarded. If it is lower BPP
(grayscale or 16 bits), it still works as expected.

## Cells

The tool exports does not export colors of landscape records that are marked as
deleted or lack vertex colors.  Only cells that are not deleted and have vertex
colors in the input plugin will be updated during import.
