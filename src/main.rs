#[derive(Clone, Copy, Debug)]
enum PBType {
    Integer64,
    String,
}

#[derive(Clone, Debug)]
struct PBField {
    name: String,
    tag: u8,
    field_type: PBType,
}

#[derive(Clone, Debug)]
struct PBMessage {
    name: String,
    fields: Vec<PBField>,
}

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
    wire_type: PBWireType,
    value: &'a [u8],
}

#[derive(Clone)]
struct ParserPosition<'a> {
    slice: &'a [u8],
    tag: u8,
    wire_type: PBWireType,
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

    p.tag = p.slice[0] >> 3;
    let wire_type_idx = (p.slice[0] & 7) as usize;
    if wire_type_idx >= IDX2WIRE_TYPE.len() {
        return Err(PBParseError::UnknownWireType(wire_type_idx));
    }
    p.wire_type = IDX2WIRE_TYPE[wire_type_idx];

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
            wire_type: p.wire_type,
        },
        p.clone()))
}

use std::str;

#[derive(Debug)]
enum ReprError {
    InvalidWireAndSchemaType,
    Utf8Error(std::str::Utf8Error),
}

fn StringRepr<'a>(wire_type: PBWireType,
                  schema_type: PBType,
                  bytes: &'a [u8])
                  -> Result<String, ReprError> {
    match (wire_type, schema_type) {
        (PBWireType::LengthDelim, PBType::String) => {
            let x = str::from_utf8(bytes);
            match x {
                Ok(y) => Ok(String::from(y)),
                Err(y) => Err(ReprError::Utf8Error(y)),
            }
        }
        (PBWireType::VarInt, PBType::Integer64) => {
          match IntegerRepr(wire_type, schema_type, bytes) {
            Ok(i) => Ok(String::from(format!("{}", i))),
            Err(e) => Err(e)
          }
        }
        _ => Err(ReprError::InvalidWireAndSchemaType),
    }
}

fn IntegerRepr<'a>(wire_type: PBWireType,
                   schema_type: PBType,
                   bytes: &'a [u8])
                   -> Result<i64, ReprError> {

    extern crate num;
    match (wire_type, schema_type) {
        (PBWireType::VarInt, PBType::Integer64) => {
            let mut pos: u8 = 0;
            let mut res: i64 = 0;
            for b in bytes {
                // I'm assuming I have an endieness problem here?
                for i in 0..7 {
                  let bit_shift = 6-i;
                  let pow_add = pos + 6 - i;
                  if ((b >> (6-i)) & 1u8) == 1u8 {
                    res += num::pow(2, (pow_add) as usize);
                  }
                }
                pos += 7;
            }
            Ok(res)
        }
        _ => Err(ReprError::InvalidWireAndSchemaType),
    }
}



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
    print!("Value = {}\n",
           StringRepr(x.wire_type, PBType::String, x.value).unwrap());

    let (x, pp) = PBParseNext(pp).unwrap();

    print!("Tag   = {}\n", x.tag);
    print!("Value = {}\n",
           StringRepr(x.wire_type, PBType::String, x.value).unwrap());

    print!("300 = {}\n",
           IntegerRepr(PBWireType::VarInt, PBType::Integer64, &[0xacu8, 0x02u8]).unwrap());
    print!("300 = string {}\n",
           StringRepr(PBWireType::VarInt, PBType::Integer64, &[0xacu8, 0x02u8]).unwrap());
}
