?:? TXT2COM Conversion utility. Vers 2.3
Copyright 1987 by Keith P. Graham
This file cannot be distributed for SALE without
express written permission from Keith P. Graham
238 Germonds Road, W. Nyack, NY 10994, (914) 623-4161
To view the contents of this file
enter the name of the file from the DOS prompt.
??  ?>?~u?6???*???c????lùP ???ֲ QR??  ??<tN<
tJ<~u2?<~t,?<#t#?<0|#?<f?<9~?<A|?<F~?<a|
????FF?¬?	? ? ?:?ZY?????ZYÊЋ??????>??16?F?H r+1???6??????Y???C;????>??1?  r??ֻ  ??C;????>??1? r????%??<u??<~u??<~t?<t?<#u??<t?:?u??Í6A?< t	??  ??????  ??6??\<t???>? u?? ?  ?  ?6??< t<0|?<9?$?????< t!< t??
??< td< t`<0|?<9?$???????< t#< t~?
??< t>< t:<0|n<9j$?׊??????< tZ< tV?
??< t< t<0|F<9B$???׊??ش ?Ë??Ǳ
????´ ?d????ƴ ?????;.??? |
?? tM?.?? ??6??< t	??  ??????  ??6 ?Z<t?? ?> u?? ?>? ? ??? ???u?+???I???6?<a|?<z?, F????Ћ??????>??16?F????Ћ?C?????>??1W?M _r??  ?????W???C;????>??1W?& _r??׻  ??C;????>??1W? _r????????<u??^?Ê^??<a|<z, :?u???I?>V?? ^?ì<t?<a|<z, G:u???^?Ã.??>? }	??  ???????????>??|????????  ??  ????  ??- ??????? ? ?O??? ??  ?Ã??? ?[??>? t?.??? ?I??>? u?H??.??A?????>??1?? ?3??%???= ??- 9?|?????????????>??1?? ????????????2?Q?O??? ? ??  ???
?	?!?6?
?*<t?????
? <?  ?!s??
??=?!s????ش@?????!?>?!?t??>? }??  á?= ~- 9?~??????  ù ???? QC9?Y?KS???>??1?>? t???<t	<
t??? ?[CY????ô ??č>?? ???u???ً?K???<;|?<Dۻ ,
<:t?,
ô? ? ?O?>:??  ??  ??6??P ?  Q??	? ? ???մ?  ?EY??ù
 ??@?  ????0???  ????0???  ????0??  ????0?~?  ?
 ??@?  ????0???  ????0???  ????0???  ?6;?P ? Q?մ?  ???	? ? ??EY??ô?? ?O?>:?ô?? ?O?>:?À<0|[?<fV?<9~?<A|L?<F~?<a|B?|0|<?|f6?|9~?|A|*?|F~?|a|?<9~,$????ج<9~,$؈:? SQRVWU??  ????=?  ? ?<tV<tU??StJ??KtN??Mtt??Rtg??Gt{??Otv<t{:}?<~ʀ>?u?? S?@?	? ?p?E?  ?մ?[C???^??&?[??? ~??x u?8 u?  MS?  ?մ?[K?y??6??q?8|?S?x u?@ ???? ?? ?W??? t?MS?ջ  ??[K?C?]_^ZY[?PVW?? I?+ˋ?G?? t????_^SR??+ӻ  ??Z[?I SR?ջ  ??Z[X?VW?? F???F+˃? t???_^SR??+ӻ  ??Z[? SR?ջ  ??Z[???SU??  ????? Q?	? ?p?YVFUQ?  ?մ???< t<t?	? ?p?EY??QY]^?  ?մ?][?PQW?? ??G? ???_YX?      If you want to write to a file enter the file name.
  If you want to print the file enter LPT1 or LPT2

  ENTER: $@                                                                  Esc=Quit,T=Write,F=Find,C=Cont,Keys=,,,,PgUp,PgDn,Home,End,Row=0000,Col=000                      KMGOIQPH!.1                                                                                   ?                                                                Enter line number:                                                                   Enter string:                                                                         ?h8?u
                          ~1F Coding and Using ANSI Music ~07
                          ~1F     in BBS Applications     ~07
                          ~1F     -------------------     ~07
                          ~0FText c 1992 by Linda R. Bloom~07


                             ~4FF1~0F   Introductory Notes~07
                             ~4FF2~0F   Notation~07
                             ~4FF3~0F   Note/Rest Values~07
                             ~4FF4~0F   Codes for ANSI-BBS Music~07
                             ~4FF5~0F   The ANSI-BBS Coding Process~07
                             ~4FF6~0F   Using ANSI-BBS Music~07
                             ~4FF7~0F   Codes for ANSI Music Messages~07
                             ~4FF8~0F   Using Music in Messages~07
                             ~4FF9~0F   Closing Comments~07

               ~1F Press <Home> at any time to return to this index. ~07






~#1Before undertaking this tutorial, you will need to obtain a means of playing
~0FANSI~07 music on your local terminal.  This will require a special utility
designed for the purpose; among these are ~4FANSIALL~07, a driver that replaces
~0FANSI.SYS~07 and adds music capability, and ~4FA-TYPE~07, a replacement for the ~0FTYPE~07
command that adds music capability to screen display.  If you are unable to
find such a utility locally, these are available on the author's BBS (~0AF9~07 for
further information).

An ~0FANSI~07 editor, such as ~4FTheDraw~07, and an ~0FASCII~07 text editor, such as
~4FUED~07, are also required to produce a finished product.

The finest computer music processor available today is a program called
~4FMelody Master v. 2.1~07.  This is the only music processing program presently
available that is capable of producing properly coded, ready-to-display
~0FANSI-BBS~07 music strings.  The program is available for download on many
national BBS systems as ~4FMELODY21.xxx~07.  Again, if you are unable to find it
locally, it is available on the author's BBS, or by mail with registration of
this tutorial (separate registration required for continued use).

You may be familiar with an ~0FANSI~07 music screen system named ~4FANSIPLAY~07.
While it is an excellent program for composing music screens for local
viewing, it is not suitable for producing music for BBS or message use.  It
includes codes and capabilities in its music strings that are specific to the
~4FANSIPLAY~07 program, and will cause undesirable effects if used for BBS or
message music strings.

~71The other end of the connection~07 - Be aware that there are only a limited number
of terminal programs that support ~0FANSI~07 music, and the only way a caller will
hear the music is by using one of these programs.  At the present time, the
following shareware terminals support ~0FANSI~07 music (version numbers are
current as of this writing):

                          ~4FQModem Test Drive v. 4.5~07
                          ~4FTelemate v.3.10         ~07
                          ~4FCI Link v. 2.52         ~07
                          ~4FSlick Terminal v. 5.7   ~07
                          ~4FQCom v. 2.93a           ~07

It is suggested that you keep these programs on-line and available for your
callers to download if you plan on adding ~0FANSI~07 music to your BBS system. It's
also a good idea to post a bulletin or news item for your callers explaining
the necessity of using a music-capable terminal program in order to receive
the music.

                                                               ~0A<Home> or <F2>~07
~#2~1FNotation~07

Music strings are written in the same format as the ~0FBASIC ~0DPLAY~07 statement.
Codes that are accepted and properly interpreted by terminal programs:

<~0DTnnn~07> <~0DOn~07> <~0DLnn~07> ~0C|~07 <~0DNote/Rest~07> <~0DSharp/Flat~07> (repeat from ~0C|~07 as required)

~0DTnnn~07       ~0FSets tempo~07; nnn represents a number between 32 and 255.  Default
           tempo is 120.

~0DOn~07         ~0FSets octave~07; n represents the octave number.  Default octave is 4.

~0DA~07 through
~0DG~07          ~0FNote~07; standard notation.

~0DP~07          ~0FPause~07; represents a rest.

~0D+~07  or  ~0D#~07   ~0FSharps a note~07; insert after the note you wish to sharp.

~0D-~07          ~0FFlats a note~07; insert after the note you wish to flat.

~0DLnn~07        ~0FLength~07; sets value of multiple notes/rests, may be dotted (see ~1FNote~07
           ~1FValues~07).

~0C*~07 Codes that are not universally accepted, and that may cause problems for
callers, are listed below.  It is ~0CSTRONGLY~07 recommended that these codes be
avoided when writing music for BBS or message use.

~0D<~07 and ~0D>~07    ~0FOctave changes~07; used in place of the ~0DOn~07 command.















                                                               ~0A<Home> or <F3>~07

~#3~1FNote/Rest Values~07

Notes and rests are coded the same way.  The only difference is that a note is
is represented by its letter name, while a rest is represented by the letter ~0DP~07.
Values are represented by the denominator of their fractional names.  Any value
may be dotted; a dot adds one-half the value of the note/rest.

          ~0FWhole note/rest~07        =   ~0D1~07
          ~0FHalf note/rest~07         =   ~0D2~07
          ~0FQuarter note/rest~07      =   ~0D4~07
          ~0FEighth note/rest~07       =   ~0D8~07
          ~0FSixteenth note/rest~07    =  ~0D16~07
          ~0FThirtysecond note/rest~07 =  ~0D32~07
          ~0FSixtyfourth note/rest~07  =  ~0D64~07

Values may be set in either of two different ways, depending on the complexity
of the music and the desired quality of the final result.

~71Method 1 - The L Command~07

The ~0DL~07 option described under ~1FNotation~07 (~0AF2~07) sets the value for all following
notes/rests until another value is specified.  Individual notes/rests with
differing values may be inserted in the string by inserting the value directly
after the note/rest.  After that note/rest plays, the default value will
return to the last preceding ~0DL~07 command.  For example, this is the first 
line of ~0BMary Had A Little Lamb~07, written using the ~0DL~07 command:

           ~0DL4EDCDEEE2DDD2EGG2~07

~0DL4~07 sets the normal note value as a quarter note.  Three half notes are in this
line of music; therefore, three of the notes are followed by the half note
value of ~0D2~07.  The rest of the note values default to the quarter note set by
the ~0DL~07 command at the start of the string.

This method is good for "quick and dirty" coding of short, simple pieces of
music that are hand-coded using a text editor.  It is not the preferred method
for more complex music, because it makes identification and debugging of
individual notes/rests more difficult, and allows for greater error in
transcription.  Because of those drawbacks, it is ~0CNOT~07 recommended that the ~0DL~07 
command be habitually used in writing ~0FANSI-BBS~07 music.

~71Method 2 - Individual Values~07

In this method, the value of each individual note/rest is inserted immediately
following the note/rest itself.  Repeating our example of ~0BMary Had A Little~07
~0BLamb~07, using the individual value method:

           ~0DE4D4C4D4E4E4E2D4D4D2E4G4G2~07

This method requires more string space, but it also allows for quick identifi-
cation of an individual note/rest, making error location and correction much
easier, and directly translates the "staff/note" pattern of music transcription
into a code form.  This makes it much better for writing and debugging complex
music.  An added benefit of this method for serious students of music is that
the mimicry and reinforcement of good transcription habits will carry over into
pen-and-ink transcription as well.

As an example of more complex music code, here is the first line of ~0BHail To~07
~0BThe Chief~07:

         ~0DO2G8A16.B32O3C8O2B16.A32G16.A32G16.E32D8C8~07

The line contains 3 octave changes and 10 note value changes.  Examining this
string should make the drawbacks of habitual ~0DL~07 command use obvious!


                                                               ~0A<Home> or <F4>~07

~#4~1FCodes for ANSI-BBS Music~07

These codes are the building blocks that create effective and properly written 
~0FANSI-BBS~07 music.  This coding "style" is designed to blend easily with the
graphics codes which are already familiar to SysOps, making the process of
adding music to graphics as painless as possible.  In addition, these codes
are recognized by just about any music-supporting terminal program, providing
musical enjoyment to the most callers with the least potential incompatibility.

~0E[0m~07      ~0FReset code~07 - Clear all previous values; "fresh start".  Used at
          the beginning and end of each string of music.

~0E[nn;ccH~07  ~0FCursor location code~07 - Move the cursor to the designated location
          on the screen and leave it there.  ~0Enn~07  represents a line number
          between 1 and 25, ~0Ecc~07 represents a column number between 1
          and 80.

          Examples:  ~0E[25;1H~07 moves the cursor to the bottom left corner of
          the screen; ~0E[1;80H~07 to the top right corner.

~0E[8m~07      ~0FInvisibility code~07 - Renders the following string invisible; none of
          the code in the string will appear on the screen.

~0E[M~07       ~0FMusic code~07 - Begin music string; code in the following string will
          be interpreted and played as music

~0E~07         ~0FEnd music code~07 - Closes the playable portion of a music string

(The left arrow is ~0FASCII 27~07.  The double sixteenth note is ~0FASCII 14~07.)

~0C*~07 Codes that are not universally accepted, and that may cause problems for
callers, are listed below.  It is ~0CSTRONGLY~07 recommended that these codes be
avoided when writing music for BBS or message use.

~0EMF/MB~07      ~0FMusic Foreground/Music Background~07; used in ~4FANSIPLAY~07 and some
           other specialty music systems.

~0EM~07<~0EN/L/S~07>   ~0FStyle code <Normal/Legato/Staccato>~07; sets the style of the string.

The first line of ~0BMary Had a Little Lamb~07 coded for use in a BBS system
screen would look like this:

          ~0E[0m[24;1H[8m[M ~0DT90E4D4C4D4E4E4E2D4D4D2E4G4G2 ~0E[0m~07

                                                               ~0A<Home> or <F5>~07

~#5~1FThe ANSI-BBS Coding Process~07

The step-by-step instructions in this section are descriptive of the process
required to code music for BBS system use.

A completed piece of ~0FANSI-BBS~07 music will consist of one or more strings of
code, each 80 characters or less in length.  It will begin with the reset
code, followed by the cursor location code, the invisibility code, the music
code, and tempo/octave/note/rest/value codes, and end with the double
sixteenth note character (~0FASCII 14~07), followed by the reset code.

Final assembly of ~0FANSI-BBS~07 music is always done using an ~0FASCII~07 text editor.
There are three basic rules to follow in assembling the final product:

~74      1)   Divide your music code into logical sections that can be coded as ~07
~74           individual strings.  For example, divide a song into phrases or   ~07
~74           "lines", and make each phrase or "line" a separate string of code.~07

There are a number of reasons for this.  First, it makes error location and
correction much easier.  Second, it helps to keep strings at a proper length.
Third, it enables you to split a song into portions and use it in creative
ways, such as playing the first part when a caller enters a door, and the
final part when he exits the door.  Last, it enables you to use only a portion
of a longer piece of music, while still retaining the integrity of and access
to the entire song.

~74      2)   Code each string as though it were a complete piece of music.     ~07

This means that each string should follow the coding pattern given above;
i.e., if any string is played individually, it will reset, locate the cursor,
set invisibility, set tempo, and reset at the end.  This will prevent any
unwanted effects from creeping in due to typographical errors.  It also makes
error location and correction much easier, as each string can be examined and
corrected individually without affecting the final product.

~74      3)   If you use a music coding program such as ~4FMusic Maker~74, you must   ~07
~74           still perform the above two steps using an ASCII text editor.     ~07

Such programs are an excellent aid, but they ~0CDO NOT~07 produce completed,
ready-for-use ~0FANSI~07 music!  The single notable exception is ~4FMelody Master~07
~4Fv. 2.1~07, which will produce ready-to-run music strings for BBS system use.
If you wish to be able to "subdivide" the song, however, you will still need to
use an ~0FASCII~07 editor to rearrange the notes into suitable phrase divisions.

To quote the old adage, "anything worth doing is worth doing well".  No SysOp
would be satisfied with running half a BBS program; why even think of settling
for half-written music?  Poorly coded music does not make a positive impres-
sion on anyone, and may actually detract from a system rather than add to it.
In other words, either use properly coded music, or don't use it at all!

We will use the song ~0BDixie~07 as an example to demonstrate the finishing
process.  A sample of this song at each stage is included in this archive
under the filenames shown in the "After completing this step" line of each
section.  It is strongly suggested that you play these samples as you proceed
through the description, to familiarize yourself with the effects of each step
in the completion process.

~71STARTING POINT - Raw Code~07

~0FNOTE~07  - If the song is hand-coded, all of the following steps can be performed
in the process of coding.  The code should be frequently saved and tested for
accuracy.

Here is the song in raw code, as produced by a program such as ~4FMusic Maker~07
(~0BRAW.BBS~07):

~0E [M ~0DT60O3C32O2A32F16F16F32G32A32A+32O3C16C16C16O2A16O3D16D16D16.C32D16.C32 ~0E~07
~0E [M ~0DD32E32F32G32A8.F32C32F8.C32O2A32O3C8.O2G32A32F8.O3C32O2A32F16F16F32G32 ~0E~07
~0E [M ~0DA32A+32O3C16C16C16O2A16O3D16D16D16.C32D16.C32D32E32F32G32A8.F32C32F8.C32 ~0E~07
~0E [M ~0DO2A32O3C8.O2G32A32F8.O3C32C32F16A16G16F16D16F8D16G8.D16G8.C16F16A16G16 ~0E~07
~0E [M ~0DF16D16E16F16.D32C16O2A16O3F16.O2A32A16G8O2A16F8.A16G8.O3D16C16O2A16 ~0E~07
~0E [M ~0DO3F16.A32G16F8O2A16F8.A16G8.O3D16C16O2A16O3F16.A32G16F8 ~0E~07

~71STEP 1~07  - The first step is to divide the code into phrases/"lines" of no more
than 55 characters each.  This particular song has familiar lyrics, which
makes the division easy:

~0DT60     O3C32 O2A32 F16  F16 F32G32 A32 A32+ O3C16 C16 C16 O2A16~0F ~07\  Line
~07(tempo)~0F Oh,   I     wish I   was    in  the  land  of  cot-ton,  ~07/  One

         ~0DO3D16 D16   D16.  C32 D16. C32 D32 E32~0F  ~07\  Line
         ~0FOld   times there are not  for-got-ten, ~07/  Two

and so on.  As you divide the lines, be sure to add the octave code to the
beginning of any lines that do not already begin with it!

After completing this step, our song looks like this (~0BSTEP1.BBS~07):

~0E [M ~0DT60O3C32O2A32F16F16F32G32A32A+32O3C16C16C16O2A16 ~0E~07
~0E [M ~0DO3D16D16D16.C32D16.C32D32E32 ~0E~07
~0E [M ~0DO3F32G32A8.F32C32F8.C32O2A32O3C8.O2G32A32F8. ~0E~07
~0E [M ~0DO3C32O2A32F16F16F32G32A32A+32O3C16C16C16O2A16 ~0E~07
~0E [M ~0DO3D16D16D16.C32D16.C32D32E32 ~0E~07
~0E [M ~0DO3F32G32A8.F32C32F8.C32O2A32O3C8.O2G32A32F8. ~0E~07
~0E [M ~0DO3C32C32F16A16G16F16D16F8D16G8.D16G8. ~0E~07
~0E [M ~0DO3C16F16A16G16F16D16E16F16. ~0E~07
~0E [M ~0DO3D32C16O2A16O3F16.O2A32A16G8 ~0E~07
~0E [M ~0DO2A16F8.A16G8.O3D16C16O2A16O3F16.A32G16F8 ~0E~07
~0E [M ~0DO2A16F8.A16G8.O3D16C16O2A16O3F16.A32G16F8 ~0E~07

In order to understand how this affects your music, try this experiment.
Using your text editor, copy line 3 of the raw code to a file, and name it
~0BRAWTEST.BBS~07.  Copy line 3 of ~0BSTEP1.BBS~07 to a file, and name it ~0BTEST.BBS~07.
Then play each test file, and compare the results.

~71STEP 2~07 - The next step is to add the tempo setting to the beginning of each
line, and add the appropriate octave code to the beginning of any lines that
do not already begin with it.  You can determine the octave code for lines
that do not begin with one by reading backward through the previous line
until you come to an octave code; that is the code to begin the line.

After completing this step, our song looks like this (~0BSTEP2.BBS~07):

~0E [M ~0DT60O3C32O2A32F16F16F32G32A32A+32O3C16C16C16O2A16 ~0E~07
~0E [M ~0DT60O3D16D16D16.C32D16.C32D32E32 ~0E~07
~0E [M ~0DT60O3F32G32A8.F32C32F8.C32O2A32O3C8.O2G32A32F8. ~0E~07
~0E [M ~0DT60O3C32O2A32F16F16F32G32A32A+32O3C16C16C16O2A16 ~0E~07
~0E [M ~0DT60O3D16D16D16.C32D16.C32D32E32 ~0E~07
~0E [M ~0DT60O3F32G32A8.F32C32F8.C32O2A32O3C8.O2G32A32F8. ~0E~07
~0E [M ~0DT60O3C32C32F16A16G16F16D16F8D16G8.D16G8. ~0E~07
~0E [M ~0DT60O3C16F16A16G16F16D16E16F16. ~0E~07
~0E [M ~0DT60O3D32C16O2A16O3F16.O2A32A16G8 ~0E~07
~0E [M ~0DT60O2A16F8.A16G8.O3D16C16O2A16O3F16.A32G16F8 ~0E~07
~0E [M ~0DT60O2A16F8.A16G8.O3D16C16O2A16O3F16.A32G16F8 ~0E~07

In order to understand how this affects your music, try this experiment.
Using your text editor, copy the last line of ~0BSTEP1.BBS~07 to a file, and name it
~0BTEST1.BBS~07.  Copy the last line of ~0BSTEP2.BBS~07 to a file, and name it ~0BTEST2.BBS~07.
Then play each test file, and compare the results.

~71STEP 3~07 - Now we are ready for the final step, adding the control codes and
making each string a complete, individual display unit.

At the beginning of each string, before the ~0E[M~07, we insert:

      ~0E[0m[24;1H[8m~07

At the end of each string, after the ~0E~07, we add:

      ~0E[0m~07

After completing this step, we have the finished product (~0BDIXIE.BBS~07):

~0E [0m[24;1H[8m[M ~0DT60O3C32O2A32F16F16F32G32A32A+32O3C16C16C16O2A16 ~0E[0m~07
~0E [0m[24;1H[8m[M ~0DT60O3D16D16D16.C32D16.C32D32E32 ~0E[0m~07
~0E [0m[24;1H[8m[M ~0DT60O3F32G32A8.F32C32F8.C32O2A32O3C8.O2G32A32F8. ~0E[0m~07
~0E [0m[24;1H[8m[M ~0DT60O3C32O2A32F16F16F32G32A32A+32O3C16C16C16O2A16 ~0E[0m~07
~0E [0m[24;1H[8m[M ~0DT60O3D16D16D16.C32D16.C32D32E32 ~0E[0m~07
~0E [0m[24;1H[8m[M ~0DT60O3F32G32A8.F32C32F8.C32O2A32O3C8.O2G32A32F8. ~0E[0m~07
~0E [0m[24;1H[8m[M ~0DT60O3C32C32F16A16G16F16D16F8D16G8.D16G8. ~0E[0m~07
~0E [0m[24;1H[8m[M ~0DT60O3C16F16A16G16F16D16E16F16. ~0E[0m~07
~0E [0m[24;1H[8m[M ~0DT60O3D32C16O2A16O3F16.O2A32A16G8 ~0E[0m~07
~0E [0m[24;1H[8m[M ~0DT60O2A16F8.A16G8.O3D16C16O2A16O3F16.A32G16F8 ~0E[0m~07
~0E [0m[24;1H[8m[M ~0DT60O2A16F8.A16G8.O3D16C16O2A16O3F16.A32G16F8 ~0E[0m~07

In order to understand how this affects your music, try this experiment.
Using your text editor, copy the last two lines of ~0BDIXIE.BBS~07 to a file, and
name it ~0BTEST3.BBS~07.  Copy the last two lines of ~0BSTEP2.BBS~07 to a file, and name
it ~0BTEST4.BBS~07.  Play ~0BTEST2.BBS~07, and then ~0BTEST3.BBS~07; observe the cursor as you
do so.
















                                                               ~0A<Home> or <F6>~07

~#6~1FUsing ANSI-BBS Music~07

The most common method of using ~0FANSI-BBS~07 music is to attach it to a screen
display.  If properly done, callers with music-capable terminals will receive
and enjoy the music as an esthetic addition to the visual display, while those
without music capability will never even know it is there.  This can be a bit
tricky, and it will require some extra time and effort; the end result,
however, is well worth the energy.

~71STEP 1~07 - The first step is to choose or design the display to which music will
be added.  Ideally, the display should not involve the last two lines of the
screen (24 and 25).  If necessary, you may bring up the chosen display in an
~0FANSI~07 editor and block-save it so that it covers lines 1 to 23 only.  This will
leave line 24 free for the cursor and for any text, prompt, etc., that may fol-
low the music when the display is used within a program, and line 25 free for
the status line required by many terminal programs.

~71STEP 2~07 - Once the display is chosen, sized, and saved as ~0FANSI~07, bring it up in
an ~0FASCII~07 text editor.  Move the cursor to the next line after the existing
code, and block copy your completed song into the file at that point.  Then
re-save the combined file under the display filename.  The music will be
invisible, and will play whenever the file is displayed.

Additional planning is required when adding music to menus or other displays
that are followed by a prompt.  Make sure that the cursor will be below the
last line of the menu display; two lines below, if possible.  The prompt will
then appear clearly, separate from the menu.

Remember, it is possible to have too much of a good thing!  There is no limit
to the number of music strings that can be attached to one screen, but more
than a few seconds' worth can be a bit irritating.  Selected strings of music,
used with restraint, can add greatly to the ambience of a system.  A long
concert at every change of display will not, and, indeed, may drive callers
away instead.

~71Other Uses~07

Uses for ~0FANSI~07 music are limited only by the imagination and creativity of the
SysOp.  It may be used with menus, newsletters, announcements, in messages, in
Email, and anywhere else that ~0FANSI~07 itself is possible.




                                                               ~0A<Home> or <F7>~07

~#7~1FCodes for ANSI Music Messages~07

When music is to be sent as a message, rather than incorporated into a system
screen, the coding scheme is a bit different.  Because BBS message systems
vary so greatly in format, and because those receiving the message may or may
not have ~0FANSI~07 and/or music capability, it is important to code the message so
that it has the least possible potential for negative effects on the reader.
To accomplish this, the following codes are used:

To begin the string -

~0E[s~07        ~0FRecords the cursor's current column position~07, which will be within
           the message system's acceptable "writing area"

~0E[8m~07       ~0FInvisibility code~07

Follow these two codes with your music string.

To end the string -

~0E[u~07        ~0FReturns the cursor~07 to the column position recorded at ~0E[s~07

~0E[A~07        ~0FReturns the cursor~07 to the beginning of the existing row

Thus, the first line of ~0BMary Had a Little Lamb~07 coded for posting as a
message would look like this:

           ~0E[s[8[M ~0DT90E4D4C4D4E4E4E2D4D4D2E4G4G2 ~0E[u[A~07
















                                                               ~0A<Home> or <F8>~07
~#8~1FUsing Music in Messages~07

The crucial consideration when using music in messages is the cursor location.
Bear in mind that any cursor location code will leave the cursor where you
have located it when the music ends.  Anything that comes after/below that
point will be overwritten by the next display. This can be quite disconcerting
if the music is in a message, and causes the message text to be obliterated
before it can be read!  Thus, rather than "guesstimating" the location for
best results in a message, we use the ~0E[r~07 code to keep the cursor where the
message system itself has put it.

In addition to allowing for the message text itself, remember that BBS message
systems use a "formatted" display for viewing messages.  By using the ~0E[r~07 and
~0E[u~07 scheme for our cursor location, we make sure that the cursor location code
leaves the cursor in a place where it will not interfere with the message
system's format.

Here is our sample song coded for message use (~0BDIXIE.MSG~07):

~0E [s[8m[M ~0DT60O3C32O2A32F16F16F32G32A32A+32O3C16C16C16O2A16 ~0E[u[A~07
~0E [s[8m[M ~0DT60O3D16D16D16.C32D16.C32D32E32 ~0E[u[A~07
~0E [s[8m[M ~0DT60O3F32G32A8.F32C32F8.C32O2A32O3C8.O2G32A32F8. ~0E[u[A~07
~0E [s[8m[M ~0DT60O3C32O2A32F16F16F32G32A32A+32O3C16C16C16O2A16 ~0E[u[A~07
~0E [s[8m[M ~0DT60O3D16D16D16.C32D16.C32D32E32 ~0E[u[A~07
~0E [s[8m[M ~0DT60O3F32G32A8.F32C32F8.C32O2A32O3C8.O2G32A32F8. ~0E[u[A~07
~0E [s[8m[M ~0DT60O3C32C32F16A16G16F16D16F8D16G8.D16G8. ~0E[u[A~07
~0E [s[8m[M ~0DT60O3C16F16A16G16F16D16E16F16. ~0E[u[A~07
~0E [s[8m[M ~0DT60O3D32C16O2A16O3F16.O2A32A16G8 ~0E[u[A~07
~0E [s[8m[M ~0DT60O2A16F8.A16G8.O3D16C16O2A16O3F16.A32G16F8 ~0E[u[A~07
~0E [s[8m[M ~0DT60O2A16F8.A16G8.O3D16C16O2A16O3F16.A32G16F8 ~0E[u[A~07

Notice that the music strings are the same; only the leading and closing codes 
are different.  If this message were to be captured and converted for use in 
an ~0FANSI-BBS~07 screen, only the leading and closing codes would require modifica-
tion, and these changes would be very simple to make using the global pattern
change function of an ~0FASCII~07 editor.








                                                               ~0A<Home> or <F9>~07

~#9It is my sincere hope that the information given here will inspire many
SysOps to add music to their systems, and to do it well.  Properly used,
music can add a wonderful touch to a BBS, and personalize it in ways that
nothing else can.

If you found this tutorial helpful, or you have any questions or comments,
please feel free to contact me by modem or mail.  If you choose to register,
you will be added to the mailing list for any future updates and news reports
concerning ~0FANSI-BBS~07 music.

~0FBY MODEM:~07   ~70 Bloomunit BBS for Homeschool Families       ~07
            ~70 (407) 687-8712, 8n1, 24 hrs/7 days, v.32bis ~07
            ~70 Running VBBS 5.52, VirtualNet Node @4070    ~07

~0FOR:~07         ~70 InterNet address:  bloomunit@delphi.com ~07

~0FBY MAIL:~07    ~70 Linda Bloom, 4986 Palm Beach Canal Road, West Palm Beach, FL 33415 ~07
~07
~0F                    Allow me to leave you with this thought:~07
      ~1F"Next to the Word of God, music is the greatest gift." Martin Luther~07
~0F                       I wish you the blessings of both!~07

                               ** End of Text **                   ~0A<Home or Esc>~07
??6u????6k??5v?AC????????9??5????57??i?
MO??6????E??3QS??;uw?g???*,???>@???#TV????02???U??????????????????S U p r ? !j!?!?!?!"H"|"?"?"##j#?#?#?#?#($t$?$%^%?%?%?%?%'&s&?&?&?&F'?'?'((?(?(?(?(?(8)?)?)?)?)?)<*?*?*++z+?+?+?+D,V,X,?,?,?,/-1-3-?-?-?-?- .Q.?.?.9/?/?/?/00x0?01'1)1{1?1?12S2U2?2?2?25373?3?3?304d4f4?4?4?4O5Q5?5?5?566d6f6?6?6?67
7\7?7?7<8?8?8?8'9)9~9?9(:*:w:?:;];?;?;<<h<j<?<=O=?=?=*>,>?>?>?>,?????$@v@x@?@AeA?A?A BMB?B?B3C?C?C?C?C?CDiDtDvD?D?D?D7E?E?EJF?F?F?FFG?G?G?G HRHTH?H?H?H$I_IaI?I?I?I"JcJ?J?JKQK?K?K?K/L1L{L?L"MVMXM?M?MEN?N?N?N!O#OhO?O?O%PYP?P?PQBQ?Q?Q?QRfR?R?R?ROS?S?S?S?S?S?S%T'T9T;T?T?T?T)U?U?UVvV?VWTW?W?W?WHX?X?XUY]Y_YaYcYeYgYiYkYmYoYqYsYuYwYyY{Y}Y?Y?Y?Y?YGZ?Z?Z5[~[?[?[?[H\?\?\=]?]?]?]^p^?^_A_C_?_?_-`[`]`?`?`Ia?a?a?a?abb[b?b?b?b?b?b?bFcHcpcrc?cd\d?d?d2e4eKeMe?e?e?effAfCfXfZf?f?fggSgrgtg?g?g?g?g?g?g?g?g?g?g?g?g?g?g?g?g?g1hShUh?h?h=i?i?i&jyj?j?j?jCk?k?k?k?k:l<l?l?l mrm?mnNn?n?nolono?opdp?p?p?p?p?p?p?p?p?p?pCqEq?q?q&r9r;r?r?r$sFsHs?s?sttVtXt?t?t?tQu?u?u?u