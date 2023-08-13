pub const COLOR_OFFSETS: [u8; 8] = [0, 4, 2, 6, 1, 5, 3, 7];

// TODO: Get missing fonts https://github.com/lattera/freebsd/tree/master/share/syscons/fonts
pub static ANSI_FONT_NAMES: [&str; 43] = [
    "IBM VGA",               // Codepage 437 English
    "IBM VGA 855",           // Codepage 1251 Cyrillic
    "IBM VGA 866",           // Maybe wrong: Russian koi8-r
    "IBM VGA 850",           // ISO-8859-2 Central European
    "IBM VGA 775",           // ISO-8859-4 Baltic wide
    "IBM VGA 866",           // Codepage 866 (c) Russian
    "IBM VGA 857",           // ISO-8859-9 Turkish
    "IBM VGA",               // Unsupported:  haik8 codepage
    "IBM VGA 862",           // ISO-8859-8 Hebrew
    "IBM VGA",               // Unsupported: Ukrainian font koi8-u
    "IBM VGA",               // Unsupported: ISO-8859-15 West European, (thin)
    "IBM VGA",               // Unsupported: ISO-8859-4 Baltic (VGA 9bit mapped)
    "IBM VGA",               // Unsupported: Russian koi8-r (b)
    "IBM VGA",               // Unsupported: ISO-8859-4 Baltic wide
    "IBM VGA",               // Unsupported: ISO-8859-5 Cyrillic
    "IBM VGA",               // Unsupported: ARMSCII-8 Character set
    "IBM VGA",               // Unsupported: ISO-8859-15 West European
    "IBM VGA 850",           // Codepage 850 Multilingual Latin I, (thin)
    "IBM VGA 850",           // Codepage 850 Multilingual Latin I
    "IBM VGA",               // Unsupported: Codepage 885 Norwegian, (thin)
    "IBM VGA",               // Unsupported: Codepage 1251 Cyrillic
    "IBM VGA",               // Unsupported: ISO-8859-7 Greek
    "IBM VGA",               // Unsupported: Russian koi8-r (c)
    "IBM VGA",               // Unsupported: ISO-8859-4 Baltic
    "IBM VGA",               // Unsupported: ISO-8859-1 West European
    "IBM VGA 866",           // Codepage 866 Russian
    "IBM VGA",               // Unsupported: Codepage 437 English, (thin)
    "IBM VGA",               // Unsupported: Codepage 866 (b) Russian
    "IBM VGA",               // Unsupported: Codepage 885 Norwegian
    "IBM VGA",               // Unsupported: Ukrainian font cp866u
    "IBM VGA",               // Unsupported: ISO-8859-1 West European, (thin)
    "IBM VGA",               // Unsupported: Codepage 1131 Belarusian, (swiss)
    "C64 PETSCII shifted",   // Commodore 64 (UPPER)
    "C64 PETSCII unshifted", // Commodore 64 (Lower)
    "C64 PETSCII shifted",   // Commodore 128 (UPPER)
    "C64 PETSCII unshifted", // Commodore 128 (Lower)
    "Atari ATASCII",         // Atari
    "Amiga P0T-NOoDLE",      // P0T NOoDLE (Amiga)
    "Amiga mOsOul",          // mO'sOul (Amiga)
    "Amiga MicroKnight+",    // MicroKnight Plus (Amiga)
    "Amiga Topaz 1+",        // Topaz Plus (Amiga)
    "Amiga MicroKnight",     // MicroKnight (Amiga)
    "Amiga Topaz 1",         // Topaz (Amiga)
];
