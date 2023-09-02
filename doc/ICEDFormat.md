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
```
Field      Bytes  Meaning
[VER]      2      LE_U16 u8 Major:u8 Minor - [00:00] atm
[Type]     1      0 - ANSI, 1 - PETSCII, 2 - ATASCII, 3 - VIEWDATA
[Width]    4      LE_U32
[Height]   4      LE_U32

```

#### END block
Keyword: 'END'

Stop parsing the PNG file.

#### SAUCE block (only 1 is valid)
Keyword: 'SAUCE'

Read content as sauce bytes.

#### Palette block (only 1 is valid)

Keyword: 'PALETTE'

```
Field      Bytes  Meaning
[NUM]      4      LE_U32 number of colors (atm only 0xFFFF colors are supported - but it may change)
                  In future (maybe): -1 means no numbers and RGB values are directly stored in the Layer    
[1]..[n]   n*4    U8 r,g,b,a values from 0..255
```

#### Bitfont Font Block
Keyword: 'FONT_{SLOT}'

```
Field      Bytes  Meaning
[NameLen]  4      LE_U32 Length of Name
[Name]     *      U8 - UTF8 encoded chars
[Length]   4      LE_U32 Data Length
[Data]     *      Font data as PSF
```

#### Layer
Keyword: 'LAYER_{SLOT}'

```
Field      Bytes  Meaning
[Title_Len]4      LE_U32 length of the utf8 title
[Title]    *      U8 - UTF8 encoded chars - Note: May only be 16 chars depending on language.
[Mode]     1      0 - normal, 1 - chars, 2 - attributes
[Color]    4      RGBA_U8 A=00 means, no color
[Flags]    4      LE_U32
                  Bit 1   : is_visible
                  Bit 2   : edit_locked
                  Bit 3   : position_locked
                  Bit 4   : has_alhpa_channel
                  Bit 5   : is_alpha_locked
[X]        4      LE_I32
[Y]        4      LE_I32
[Width]    4      LE_U32
[Height]   4      LE_U32
[DataLen]  8      LE_U64 Length of Data
[Data]     *      Ansi encoded data
```
