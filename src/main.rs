#[derive(Clone, Copy, Debug)]
enum PBWireType {
    VarInt,
    Fixed64,
    LengthDelim,
    Fixed32,
    Reserved,
}


struct TagValue<'a> {
    tag: u8,
    value: &'a [u8],
}

#[derive(Clone)]
struct ParserPosition {
    position: usize,
    tag: u8,
    wire_type: PBWireType,
    cur_idx: u8,
    max_idx: u8,
}

fn PBParseNext<'a>(s: &'a [u8], pos: ParserPosition) -> (TagValue<'a>, ParserPosition) {
    let idx2wireType = vec![PBWireType::VarInt,
                            PBWireType::Fixed64,
                            PBWireType::LengthDelim,
                            PBWireType::Reserved,
                            PBWireType::Reserved,
                            PBWireType::Fixed32];

    let mut p = pos.clone();

    if p.cur_idx == p.max_idx {
        p.tag = s[p.position] >> 3;
        p.wire_type = idx2wireType[(s[p.position] & 7) as usize];
        p.position += 1;
        p.cur_idx = 0;
        p.max_idx = 0;
    }
    else {
      p.cur_idx += 1;
    }
    let mut value = match p.wire_type {
        PBWireType::LengthDelim => {
            let offset = s[p.position] as usize;
            let start = p.position + 1;
            let end = p.position + offset;
            p.position = end + 1;
            &s[start..end + 1]
        }
        _ => &[],
    };
    (TagValue {
         value: value,
         tag: p.tag,
     },
     p.clone())
}

use std::str;

fn main() {
    /*
     * Sample ProtocolBuffer message for schema:
     *
     *     message Test2 {
     *       required string a = 1;
     *       required string b = 2;
     *     }
     *
     * with a = test, b = testing
     *
     * @see https://developers.google.com/protocol-buffers/docs/encoding
     */
    let msg = vec![0x12, 0x07, 0x74, 0x65, 0x73, 0x74, 0x69, 0x6e, 0x67,
                   0x0a, 0x04, 0x74, 0x65, 0x73, 0x74];

    let (mut x, mut pp) = PBParseNext(msg.as_slice(),
                              ParserPosition {
                                  position: 0,
                                  tag: 0,
                                  wire_type: PBWireType::Reserved,
                                  cur_idx: 0,
                                  max_idx: 0,
                              });

    print!("Tag   = {}\n", x.tag);
    print!("Value = {}\n", str::from_utf8(x.value).unwrap());

    let (x, pp) = PBParseNext(msg.as_slice(), pp);

    print!("Tag   = {}\n", x.tag);
    print!("Value = {}\n", str::from_utf8(x.value).unwrap());
}
