# Icy Draw file format

I dump the Mystic Draw format completely. Fortunately there are 0 files in this format.
I learned too much about modern ANSIs to restart.
Let's look.

## Goals

- Every supported format should be represented. Including tundra.
- Be compatible to Sauce/XBin models as much as possible.
- Allow previews in the file explorer at best without much need for icy draw installed.
- Try to be extensible

## Format

It's a png file showing the file contents in some form augmented with base64 encoded ztxt data blocks.

### Header

Keyword: 'ICED'

|Field      |Bytes  | Type | Meaning
|-----------|-------|------|----------------------------------
|[VER]      |2      |LE_U16| u8 Major:u8 Minor - [00:00] atm
|[Type]     |4      |LE_U32| 0 - CP437/8BIT_ANSI, 1 - UNICODE ANSI, 2 - PETSCII, 3 - ATASCII, 4 - VIEWDATA
|[Mode]     |2      |LE_U16| Mode flags[^1]

[^1] Mode flags for CP437
|Flag | Meaning
|-----|----------
| 0   | True Color
| 1   | 16c
| 2   | ICE
| 3   | 256 Colors
| 4   | XB extended font + blink
| 5   | XB extended font ice

#### TYPE SPECFIC DATA

For "character buffer" types (ansi/ascii):
|Field      |Bytes  | Type |Meaning
|-----------|-------|------|--
| Width     | 4     |LE_U32| width in chars
| Height    | 4     |LE_U32| height in chars

#### END block

Keyword: 'END'

Stop parsing the PNG file.

#### SAUCE block (only 1 is valid)

Keyword: 'SAUCE'

Read content as sauce bytes.

#### Bitfont Font Block

Keyword: 'FONT_{SLOT}'
|Field      |Bytes  | Type | Meaning
|-----------|-------|------|-------------------
|NameLen    |4      |LE_U32| Length of Name
|Name       |*      |UTF8  | encoded chars
|Data       |*      |*     | Font data as PSF

#### Layer

Keyword: 'LAYER_{SLOT}'

|Field        |Bytes   | Type | Meaning
|-------------|--------|------|-------------------------------
|Title_Len    |4       |LE_U32| length of the utf8 title
|Title        |*       |UTF8  | encoded chars
|Role         |1       |U8    | 0 - normal, 1 - image (data contains rgba image data)
|Extra        |4       |LE_U32| unused
|Mode         |1       |U8    | 0 - normal, 1 - chars, 2 - attributes
|Color        |4       |LE_U32| RGBA_U8 A=00 means, no color
|Flags        |4       |LE_U32|
|             |        |Bit 1   : is_visible
|             |        |Bit 2   : edit_locked
|             |        |Bit 3   : position_locked
|             |        |Bit 4   : has_alhpa_channel
|             |        |Bit 5   : is_alpha_locked
|X            |4       |LE_I32 |
|Y            |4       |LE_I32 |
|Width        |4       |LE_U32 |
|Height       |4       |LE_U32 |
|DataLen      |8       |LE_U64 | Length of Data
|Data         |*       |*      | Buffer Data

The buffer data is the internal icy_draw buffer format. That covers ansi (cp437 and unicode), petscii, atari, viewdata.

[attribute]  2  LE_U16
-> indicates invisible then it ends here

Char         4  UTF_32 char
Fg           4  LE_U32 Foreground
Bg           4  LE_U32 Background
Font_Page    2  LE_U16

If attribute is marked with attribute::SHORT_DATA (0b0100_0000_0000_0000) then char/fg/bg/Font_Page is represented as U8. Even if the data is crunched file sizes are halved with this change. That's fully the case in cp437, petscii, atari, viewdata. In Unicode files for the majority of chars. There may be additional flag indicators in the future.

Image data:
|Field        | Bytes   | Type   | Meaning
|-------------|---------|--------|-------------------------------
| Width       | 4       | LE_U32 | Width
| Height      | 4       | LE_U32 | Height
| X-Scale     | 4       | LE_U32 | vertical scale
| Y-Scale     | 4       | LE_U32 | horizontal scale
| RGBA-Data   | *       | U8 RGBA| encoded data width * height * 4
