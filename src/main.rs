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
struct ParserPosition<'a> {
    slice: &'a [u8],
    tag: u8,
    wire_type: PBWireType,
    cur_idx: u8,
    max_idx: u8,
}

trait InitParserPosition<'a> {
    fn init(msg_slice: &'a [u8]) -> ParserPosition<'a>;
}

impl<'a> InitParserPosition<'a> for ParserPosition<'a> {
    fn init(msg_slice: &'a [u8]) -> ParserPosition<'a> {
        ParserPosition {
            slice: msg_slice,
            tag: 0,
            wire_type: PBWireType::Reserved,
            cur_idx: 0,
            max_idx: 0,
        }
    }
}

static IDX2WIRE_TYPE: &'static [PBWireType; 6] = &[PBWireType::VarInt,
                                                   PBWireType::Fixed64,
                                                   PBWireType::LengthDelim,
                                                   PBWireType::Reserved,
                                                   PBWireType::Reserved,
                                                   PBWireType::Fixed32];
#[derive(Clone, Copy, Debug)]
enum PBParseError {
    UnknownWireType(usize),
}

fn PBParseNext<'a>(pos: ParserPosition<'a>)
                   -> Result<(TagValue<'a>, ParserPosition<'a>), PBParseError> {
    let mut p = pos.clone();

    if p.cur_idx == p.max_idx {
        p.tag = p.slice[0] >> 3;
        let wire_type_idx = (p.slice[0] & 7) as usize;
        if wire_type_idx >= IDX2WIRE_TYPE.len() {
            return Err(PBParseError::UnknownWireType(wire_type_idx));
        }
        p.wire_type = IDX2WIRE_TYPE[wire_type_idx];
        p.cur_idx = 0;
        p.max_idx = 0;
    } else {
        p.cur_idx += 1;
    }
    let mut value = match p.wire_type {
        PBWireType::LengthDelim => {
            let offset = p.slice[1] as usize;
            let start = 2;
            let end = 2 + offset;
            let x = &p.slice[start..end];
            p.slice = &p.slice[end..];
            x
        }
        _ => &[],
    };
    Ok((TagValue {
            value: value,
            tag: p.tag,
        },
        p.clone()))
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
    let msg = vec![0x12, 0x07, 0x74, 0x65, 0x73, 0x74, 0x69, 0x6e, 0x67, 0x0a, 0x04, 0x74, 0x65,
                   0x73, 0x74];
    let msg_slice = msg.as_slice();

    let (mut x, mut pp) = PBParseNext(ParserPosition::init(msg_slice)).unwrap();

    print!("Tag   = {}\n", x.tag);
    print!("Value = {}\n", str::from_utf8(x.value).unwrap());

    let (x, pp) = PBParseNext(pp).unwrap();

    print!("Tag   = {}\n", x.tag);
    print!("Value = {}\n", str::from_utf8(x.value).unwrap());
}
