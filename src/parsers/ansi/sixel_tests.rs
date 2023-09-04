use std::{thread, time::Duration};

use crate::{
    ansi::Parser,
    parsers::{create_buffer, update_buffer},
    Buffer, Position,
};

fn update_sixels(buf: &mut Buffer) {
    while !buf.sixel_threads.is_empty() {
        buf.update_sixel_threads();
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn test_simple_sixel() {
    let (mut buf, _) = create_buffer(&mut Parser::default(), b"\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    update_sixels(&mut buf);
    assert_eq!(1, buf.layers[0].sixels.len());
    assert_eq!(2, buf.layers[0].sixels[0].vertical_scale);
    assert_eq!(1, buf.layers[0].sixels[0].horizontal_scale);
    assert_eq!(Position::new(0, 0), buf.layers[0].sixels[0].position);
    assert_eq!(14, buf.layers[0].sixels[0].get_width());
    assert_eq!(12, buf.layers[0].sixels[0].get_height());
}

#[test]
fn test_simple_position_sixel() {
    let (mut buf, _) = create_buffer(&mut Parser::default(), b"\x1B[4;13H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    update_sixels(&mut buf);
    let sixels = &buf.layers[0].sixels;

    assert_eq!(1, sixels.len());
    assert_eq!(Position::new(12, 3), sixels[0].position);
    assert_eq!(14, sixels[0].get_width());
    assert_eq!(12, sixels[0].get_height());
}

#[test]
fn test_overwrite_sixel() {
    let (mut buf, mut caret) = create_buffer(&mut Parser::default(), b"\x1B[4;13H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[4;13H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[4;13H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[4;13H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[4;13H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[4;13H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    update_sixels(&mut buf);

    let sixels = &buf.layers[0].sixels;
    assert_eq!(1, sixels.len());
    assert_eq!(Position::new(12, 3), sixels[0].position);
    assert_eq!(14, sixels[0].get_width());
    assert_eq!(12, sixels[0].get_height());
}

#[test]
fn test_overwrite_multiple_sixels() {
    let (mut buf, mut caret) = create_buffer(&mut Parser::default(), b"\x1B[0;0H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[5;5H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[10;10H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    for _ in 0..10 {
        update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[0;0H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
        update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[5;5H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
        update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[10;10H\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\");
    }
    update_sixels(&mut buf);

    let sixels = &buf.layers[0].sixels;
    assert_eq!(3, sixels.len());
    assert_eq!(Position::new(0, 0), sixels[0].position);
    assert_eq!(Position::new(4, 4), sixels[1].position);
    assert_eq!(Position::new(9, 9), sixels[2].position);

    (0..sixels.len()).for_each(|i| {
        assert_eq!(14, sixels[i].get_width());
        assert_eq!(12, sixels[i].get_height());
    });
}

#[test]
fn test_chess_update() {
    let (mut buf, mut caret) = create_buffer(&mut Parser::default(), b"");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[8;1f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2!10~@!6?BB!7?BB!5?@!12~$#0!10?}!6~{{!7~{{!5~}-#2!10~}{ww_#3???!9W#2???oww{!12~$#0!10?@BFF^~~~!9f~~~NFFB-#2!15~#0!15~#2!16~-!12~NNB#3!4?!7C#2!4?FNN!13~$#0!12?oo{!4~!7z!4~woo-#2!9~@!25?@!10~$#0!9?}!25~}-#2!10@#0!25@#2!10@#1@-\x1B\\\x1B[6;1f\x1B[8;7f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[8;13f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2!20~b@!5?`!18~$#0!20?[}vbbv~]$#3!22?G[[G-#2!14~NFB@@#3!4?ww#2!4?@@BF^!12~$#0!14?ow{}}!4~FF!4~}}{w_-#2!13~_#3!6?BBB^^BBB#2!5?_!12~$#0!13?^!6~{{{__{{{!5~^-#2!15~KG#3??C!8cC#2??CM!13~$#0!15?rv~~z!8Zz~~zp-#2!4~^NFF!5B@@#3!7?!4_#2!8?@BBFFNN^!4~$#0!4?_oww!5{}}!7~!4^!8~}{{wwoo_-#2@@@#0!15@#3!11@#0!13@#2@@@#1@-\x1B\\\x1B[6;1f\x1B[8;19f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1!6~NNFFNN~BB@@BB~o!4?o~~B!4@b~!5N^!5~$#0!6?oowwoo?{{ee{{?N~xx~N??{}ee}[?!5o_$#3!15?WW!5?EE!6?WW-#1!6~wo???G~~!4?B~~!4?~~N!4?~~^???oow!5~$#0!6?FN{{~v??!4~{??!4~??o!4~??_~~{KNF$#3!8?BB!27?BB-#1!8~{!4?B!6?@!11?@!4?w!8~$#0!8?B!4~{!6~}!11~}!4~F-#1!10~{w_#3??CC!4E!4FEEECC#1???_ow}!9~$#0!10?BF^~~zz!4x!4wxxxzz~~~^NF@-#1!12~B#3!4?!13G#1???B!12~$#0!12?{!4~!13v~~~{-#1!12@#0!22@#1!11@#2@-\x1B\\\x1B[6;1f\x1B[8;25f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2!10~@!6?BB!7?BB!5?@!12~$#0!10?}!6~{{!7~{{!5~}-#2!10~}{ww_#3???!9W#2???oww{!12~$#0!10?@BFF^~~~!9f~~~NFFB-#2!15~#0!15~#2!16~-!12~NNB#3!4?!7C#2!4?FNN!13~$#0!12?oo{!4~!7z!4~woo-#2!9~@!25?@!10~$#0!9?}!25~}-#2!10@#0!25@#2!10@#1@-\x1B\\\x1B[6;1f\x1B[8;31f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[8;37f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2!20~xX??Xx!20~$#0!20?Ee~~eE-#2!7~^NF!6BFFN@!6?@FF!6BFN^!8~$#0!7?_ow{[[{[{wwo}~~rr~~}ww{{[{[{wo_$#3!11?__?_!7?KK!7?_?_-#2!7~#0~~pM!5~}~zn!6~^z~}!4~}@~~#2!8~$#3!9?Mp!5?@?CO!6?_C?@!4?@}-#2!8~}w_#3@AG_??_?Od?_??_AG_??_GA@$#0!8?@F^}|v^~~^~nY~^~~^|v^~~^v|]NB$#2!34?_o{!9~-!11~w#3???!15G#2???o!12~$#0!11?F~~~!15v~~~N-#2!12@#0!21@#2!12@#1@-\x1B\\\x1B[6;1f\x1B[8;43f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[6;1f\x1B[9;49f 8\x1B[?25l\x1B[10;1f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1!18~FB@???@BF!19~$#0!18?w{}~~~}{w-#1!15~NFB@!7?@BFN!16~$#0!15?ow{}!7~}{wo-#1!15~}wO!9?Ow{!16~$#0!15?@Fn!9~nFB-#1!13~NB@!13?@BFN!13~$#0!13?o{}!13~}{wo-#1!12~#0!22~#1!12~-#3@#1!11@#0!22@#1!11@#2@-\x1B\\\x1B[6;1f\x1B[10;7f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2!18~FB@???@BF!19~$#0!18?w{}~~~}{w-#2!15~NFB@!7?@BFN!16~$#0!15?ow{}!7~}{wo-#2!15~}wO!9?Ow{!16~$#0!15?@Fn!9~nFB-#2!13~NB@!13?@BFN!13~$#0!13?o{}!13~}{wo-#2!12~#0!22~#2!12~-#3@#2!11@#0!22@#2!11@#1@-\x1B\\\x1B[6;1f\x1B[10;13f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1!18~FB@???@BF!19~$#0!18?w{}~~~}{w-#1!15~NFB@!7?@BFN!16~$#0!15?ow{}!7~}{wo-#1!15~}wO!9?Ow{!16~$#0!15?@Fn!9~nFB-#1!13~NB@!13?@BFN!13~$#0!13?o{}!13~}{wo-#1!12~#0!22~#1!12~-#3@#1!11@#0!22@#1!11@#2@-\x1B\\\x1B[6;1f\x1B[10;19f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[10;25f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[10;31f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2!18~FB@???@BF!19~$#0!18?w{}~~~}{w-#2!15~NFB@!7?@BFN!16~$#0!15?ow{}!7~}{wo-#2!15~}wO!9?Ow{!16~$#0!15?@Fn!9~nFB-#2!13~NB@!13?@BFN!13~$#0!13?o{}!13~}{wo-#2!12~#0!22~#2!12~-#3@#2!11@#0!22@#2!11@#1@-\x1B\\\x1B[6;1f\x1B[10;37f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1!18~FB@???@BF!19~$#0!18?w{}~~~}{w-#1!15~NFB@!7?@BFN!16~$#0!15?ow{}!7~}{wo-#1!15~}wO!9?Ow{!16~$#0!15?@Fn!9~nFB-#1!13~NB@!13?@BFN!13~$#0!13?o{}!13~}{wo-#1!12~#0!22~#1!12~-#3@#1!11@#0!22@#1!11@#2@-\x1B\\\x1B[6;1f\x1B[10;43f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2!18~FB@???@BF!19~$#0!18?w{}~~~}{w-#2!15~NFB@!7?@BFN!16~$#0!15?ow{}!7~}{wo-#2!15~}wO!9?Ow{!16~$#0!15?@Fn!9~nFB-#2!13~NB@!13?@BFN!13~$#0!13?o{}!13~}{wo-#2!12~#0!22~#2!12~-#3@#2!11@#0!22@#2!11@#1@-\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[6;1f\x1B[11;49f 7\x1B[?25l\x1B[12;1f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[12;7f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[12;13f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2!14~^??@BB@???@!4N^^!15~$#0!14?_~z}{{}~~~}!4o__$#3!16?C-#2!12~NB#3?_w[W!9?C?G?_$#0!12?o{~^Fbf!9~z~v~]{wo$#2!32?@BFN!10~-!8~^FB#3!23?G#2?@N!8~$#0!8?_w{!23~v~}o-#2!8~w__!6?_w{KE#3!13?A_#2?@!7~$#0!8?F^^{!5~^FBrx!13~|^~}$#3!11?B-#2!13~}}~~F@#3!17?I#2??!7~$#0!13?@@??w}!17~t~~-#2!17@#0!22@#2!6@#1@-\x1B\\\x1B[6;1f\x1B[12;19f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1!20~b@!5?`!18~$#0!20?[}vbbv~]$#3!22?G[[G-#1!14~NFB@@#3!4?ww#1!4?@@BF^!12~$#0!14?ow{}}!4~FF!4~}}{w_-#1!13~_#3!6?BBB^^BBB#1!5?_!12~$#0!13?^!6~{{{__{{{!5~^-#1!15~KG#3??C!8cC#1??CM!13~$#0!15?rv~~z!8Zz~~zp-#1!4~^NFF!5B@@#3!7?!4_#1!8?@BBFFNN^!4~$#0!4?_oww!5{}}!7~!4^!8~}{{wwoo_-#1@@@#0!15@#3!11@#0!13@#1@@@#2@-\x1B\\\x1B[6;1f\x1B[12;25f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2!18~FB@???@BF!19~$#0!18?w{}~~~}{w-#2!15~NFB@!7?@BFN!16~$#0!15?ow{}!7~}{wo-#2!15~}wO!9?Ow{!16~$#0!15?@Fn!9~nFB-#2!13~NB@!13?@BFN!13~$#0!13?o{}!13~}{wo-#2!12~#0!22~#2!12~-#3@#2!11@#0!22@#2!11@#1@-\x1B\\\x1B[6;1f\x1B[12;31f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[12;37f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[12;43f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[6;1f\x1B[13;49f 6\x1B[?25l\x1B[14;1f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[14;7f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[14;13f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[14;19f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[14;25f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[14;31f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[14;37f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[14;43f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[6;1f\x1B[15;49f 5\x1B[?25l\x1B[16;1f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[16;7f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[16;13f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[16;19f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1!18~FB@???@BF!19~$#0!18?wkEBBBEkw$#3!19?Ow{{{wO-#1!15~NFB@!7?@BFN!16~$#0!15?ow[MFB???BFM[wo$#3!17?_ow{~~~{wo_-#1!15~}wO!9?Ow{!16~$#0!15?@Fm{wO???Ow{mFB$#3!17?@BFn~~~nFB@-#1!13~NB@!13?@BFN!13~$#0!13?o{MFB@!7?@BFEKwo$#3!15?ow{}!7~}{wwo-#1!12~#0~b!18_b~#1!12~$#3!13?[!18^[-#1!12@#0!22@#1!11@#2@-\x1B\\\x1B[6;1f\x1B[16;25f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2!20~b@!5?`!18~$#0!20?[}vbbv~]$#3!22?G[[G-#2!14~NFB@@!10?@@BF^!12~$#0!14?ow{]MFFBBxxBBFFM]{w_$#3!17?_oww{{EE{{wwo_-#2!13~`#3??N^~~{{{__{{{~^N#2??_!12~$#0!13?]~~o_??BBB^^BBB?_o~~^-#2!15~KG#3??C!8cC#2??CM!13~$#0!15?rv~~z!8Zz~~zp-#2!4~^NFF!5B@@!19?@BBFFNN^!4~$#0!4?_oww[[KKKMMN!6F!4f!7FNMK[wwoo_$#3!8?__!6o!6w!4W!7wooo_-#2@@@#0@@@#3!12@#0!11@#3!10@#0@@@#2@@@#1@-\x1B\\\x1B[6;1f\x1B[16;31f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[16;37f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[16;43f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[6;1f\x1B[17;49f 4\x1B[?25l\x1B[18;1f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[18;7f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[18;13f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[18;19f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[18;25f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[18;31f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2!14~^??@BB@???@!5N^!15~$#0!14?_~z]{[MFN~}!5o_$#3!16?C_?_owo-#2!12~NB#3o[Fbf!4~}^M~}}wwo#2@BFN!10~$#0!12?o{Nbw[W!4?@_p?@@FFN}{wo-#2!8~^FB!25?@N!8~$#0!8?_w[NB@!6?_ow~~@!7?@F^}o$#3!10?_o{}!6~^NF??}!7~}w_-#2!8~w__!6?_w{KE#0!14?N~}$!8?F]Wro_oow[FBrx~N@#2!13?@!7~$#3!9?@FKN^NNFB!5?o}!11~o-#2!13~}}~~F@#3?W]!15^#2??!7~$#0!13?@@??w}~f`!15_~~-#2!17@#0!22@#2!6@#1@-\x1B\\\x1B[6;1f\x1B[18;37f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[18;43f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[6;1f\x1B[19;49f 3\x1B[?25l\x1B[20;1f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2!18~FB@???@BF!19~$#0!18?wkEBBBEkw$#3!19?Ow{{{wO-#2!15~NFB@!7?@BFN!16~$#0!15?ow[MFB???BFM[wo$#3!17?_ow{~~~{wo_-#2!15~}wO!9?Ow{!16~$#0!15?@Fm{wO???Ow{mFB$#3!17?@BFn~~~nFB@-#2!13~NB@!13?@BFN!13~$#0!13?o{MFB@!7?@BFEKwo$#3!15?ow{}!7~}{wwo-#2!12~#0~b!18_b~#2!12~$#3!13?[!18^[-#2!12@#0!22@#2!11@#1@-\x1B\\");
    update_sixels(&mut buf);

    {
        let sixels = &buf.layers[0].sixels;

        assert_eq!(49, sixels.len());
        (0..sixels.len()).for_each(|i| {
            assert_eq!(31, sixels[i].get_height());
            assert_eq!(46, sixels[i].get_width());
        });
    }

    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[6;1f\x1B[15;49f 5\x1B[?25l\x1B[16;1f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[16;7f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[16;13f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[16;19f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1!18~FB@???@BF!19~$#0!18?wkEBBBEkw$#3!19?Ow{{{wO-#1!15~NFB@!7?@BFN!16~$#0!15?ow[MFB???BFM[wo$#3!17?_ow{~~~{wo_-#1!15~}wO!9?Ow{!16~$#0!15?@Fm{wO???Ow{mFB$#3!17?@BFn~~~nFB@-#1!13~NB@!13?@BFN!13~$#0!13?o{MFB@!7?@BFEKwo$#3!15?ow{}!7~}{wwo-#1!12~#0~b!18_b~#1!12~$#3!13?[!18^[-#1!12@#0!22@#1!11@#2@-\x1B\\\x1B[6;1f\x1B[16;25f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2!20~b@!5?`!18~$#0!20?[}vbbv~]$#3!22?G[[G-#2!14~NFB@@!10?@@BF^!12~$#0!14?ow{]MFFBBxxBBFFM]{w_$#3!17?_oww{{EE{{wwo_-#2!13~`#3??N^~~{{{__{{{~^N#2??_!12~$#0!13?]~~o_??BBB^^BBB?_o~~^-#2!15~KG#3??C!8cC#2??CM!13~$#0!15?rv~~z!8Zz~~zp-#2!4~^NFF!5B@@!19?@BBFFNN^!4~$#0!4?_oww[[KKKMMN!6F!4f!7FNMK[wwoo_$#3!8?__!6o!6w!4W!7wooo_-#2@@@#0@@@#3!12@#0!11@#3!10@#0@@@#2@@@#1@-\x1B\\\x1B[6;1f\x1B[16;31f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[16;37f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[16;43f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[6;1f\x1B[17;49f 4\x1B[?25l\x1B[18;1f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[18;7f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[18;13f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[18;19f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\\x1B[6;1f\x1B[18;25f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[18;31f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2!14~^??@BB@???@!5N^!15~$#0!14?_~z]{[MFN~}!5o_$#3!16?C_?_owo-#2!12~NB#3o[Fbf!4~}^M~}}wwo#2@BFN!10~$#0!12?o{Nbw[W!4?@_p?@@FFN}{wo-#2!8~^FB!25?@N!8~$#0!8?_w[NB@!6?_ow~~@!7?@F^}o$#3!10?_o{}!6~^NF??}!7~}w_-#2!8~w__!6?_w{KE#0!14?N~}$!8?F]Wro_oow[FBrx~N@#2!13?@!7~$#3!9?@FKN^NNFB!5?o}!11~o-#2!13~}}~~F@#3?W]!15^#2??!7~$#0!13?@@??w}~f`!15_~~-#2!17@#0!22@#2!6@#1@-\x1B\\\x1B[6;1f\x1B[18;37f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#1}!44~}$#3@#2!44?@-#1!46~-!46~-!46~-!46~-#0@#1!44@#0@-\x1B\\\x1B[6;1f\x1B[18;43f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2}!44~}$#3@#1!44?@-#2!46~-!46~-!46~-!46~-#3@#2!44@#0@-\x1B\\");
    update_buffer(&mut buf, &mut caret, &mut Parser::default(), b"\x1B[6;1f\x1B[19;49f 3\x1B[?25l\x1B[20;1f\x1BP0;1;0q\"1;1;46;31#0;2;0;0;0#1;2;93;11;14#2;2;94;89;69#3;2;100;100;100#2!18~FB@???@BF!19~$#0!18?wkEBBBEkw$#3!19?Ow{{{wO-#2!15~NFB@!7?@BFN!16~$#0!15?ow[MFB???BFM[wo$#3!17?_ow{~~~{wo_-#2!15~}wO!9?Ow{!16~$#0!15?@Fm{wO???Ow{mFB$#3!17?@BFn~~~nFB@-#2!13~NB@!13?@BFN!13~$#0!13?o{MFB@!7?@BFEKwo$#3!15?ow{}!7~}{wwo-#2!12~#0~b!18_b~#2!12~$#3!13?[!18^[-#2!12@#0!22@#2!11@#1@-\x1B\\");
    update_sixels(&mut buf);

    let sixels = &buf.layers[0].sixels;
    assert_eq!(49, sixels.len());
    (0..sixels.len()).for_each(|i| {
        assert_eq!(31, sixels[i].get_height());
        assert_eq!(46, sixels[i].get_width());
    });
}

#[test]
fn test_macro_sixels() {
    let mut parser: Parser = Parser::default();
    let (mut buf, mut caret) = create_buffer(&mut parser, b"\x1BP11;0;0!zq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\\r\n");
    update_buffer(&mut buf, &mut caret, &mut parser, b"\x1BP12;0;0!zq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\\r\n");
    assert_eq!(2, parser.macros.len());
    update_buffer(
        &mut buf,
        &mut caret,
        &mut parser,
        b"\x1B[10;59f\x1BP\x1B[11*z\x1B\\",
    );
    update_sixels(&mut buf);
    {
        assert_eq!(1, buf.layers[0].sixels.len());
    }
    update_buffer(
        &mut buf,
        &mut caret,
        &mut parser,
        b"\x1B[0;59f\x1BP\x1B[11*z\x1B\\",
    );
    update_sixels(&mut buf);

    {
        assert_eq!(2, buf.layers[0].sixels.len());
    }
}

#[test]
fn test_simple_sixel2() {
    let mut parser: Parser = Parser::default();
    let (mut buf, mut caret) = create_buffer(&mut parser, b"\x1B[1;1H");
    update_sixels(&mut buf);

    update_buffer(&mut buf, &mut caret, &mut parser, b"test\n\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\\n\r");
    update_buffer(&mut buf, &mut caret, &mut parser, b"test\n\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\\n\r");
    update_buffer(&mut buf, &mut caret, &mut parser, b"test\n\x1BPq#0;2;0;0;0#1;2;100;100;0#2;2;0;100;0#1~~@@vv@@~~@@~~$43#2??}}GG}}??}}??-#1!14@\x1B\\\n\r");
}

#[test]
fn test_sixel_raster_attributes() {
    let (mut buf, _) = create_buffer(&mut Parser::default(), b"\x1BPq\"2;3;6;8\x1B\\");
    update_sixels(&mut buf);

    assert_eq!(1, buf.layers[0].sixels.len());
    assert_eq!(3, buf.layers[0].sixels[0].horizontal_scale);
    assert_eq!(8, buf.layers[0].sixels[0].get_height());
    assert_eq!(6, buf.layers[0].sixels[0].get_width());

    assert_eq!(6, buf.layers[0].sixels[0].get_width());
    assert_eq!(8, buf.layers[0].sixels[0].get_height());
}
