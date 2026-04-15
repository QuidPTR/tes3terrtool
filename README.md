# Description

A tiny tool to import/export vertex color data into/from TES3 ESM/ESP files. It
is designed to enable easy vertex color editing using external editors.

**Warning**: preliminary, rough and untested.


# Usage

The tool allows to both export and import vertex color data:

    * Export lets you specify a source ESM/ESP file and a target image to which
      the vertex colors will be dumped.

    * Import lets you specify a source ESM/ESP file and a source image holding
      vertex color data (for instance, one you previously exported from the
      source ESM/ESP), and will create a new ESM/ESP that is (ideally) identical
      to the original except the vertex colors will be taken from the image.


# Examples:

Exporting:

```sh
tes3terrtool export-vcol --input-esm Morrowind.esm --output-image Morrowind-vcol.bmp
```

Importing:

```sh
tes3terrtool import-vcol --input-esm Morrowind.esm --input-image Morrowind-vcol-edited.bmp --output-esm Morrowind-edited.ems
```
