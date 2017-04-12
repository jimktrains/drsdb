#[derive(Clone)]
enum PBType {
    Double,
    Float,
    Int32,
    Int64,
    Uint32,
    Uint64,
    Sint32,
    sInt64,
    Fixed32,
    Fixed64,
    Sfixed32,
    Sfixed64,
    Bool,
    String,
    Bytes,
}

struct PBField {
    name: String,
    tag: u8,
    field_type: PBType,
}

struct PBMessage {
    name: String,
    fields: Vec<PBField>,
}

#[derive(Debug)]
#[derive(Clone)]
enum PBWireType {
    VarInt,
    Fixed64,
    LengthDelim,
    Fixed32,
    Reserved,
}


struct TagValue {
    tag: u8,
    value: Vec<u8>,
}

#[derive(Clone)]
struct ParserPosition {
    position: usize,
    tag: u8,
    wire_type: PBWireType,
    cur_idx: u8,
    max_idx: u8,
}

fn PBParseNext(s: Vec<u8>, pos: ParserPosition) -> (TagValue, ParserPosition) {
    let idx2wireType = vec![PBWireType::VarInt,
                            PBWireType::Fixed64,
                            PBWireType::LengthDelim,
                            PBWireType::Reserved,
                            PBWireType::Reserved,
                            PBWireType::Fixed32];

    let mut p = pos.clone();

    if p.cur_idx == p.max_idx {
        p.tag = s[p.position] >> 3;
        p.wire_type = idx2wireType[(s[p.position] & 7) as usize].clone();
        p.position += 1;
        p.cur_idx = 0;
        p.max_idx = 0;
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
    let mut v = Vec::new();
    v.extend_from_slice(value);
    (TagValue {
         value: v.clone(),
         tag: p.tag,
     },
     p.clone())
}

fn PBBytesToString(field_type: PBType, bytes: Vec<u8>) -> String {
    match field_type {
        PBType::String => String::from_utf8(bytes).unwrap(),
        _ => String::from("Not Implemented"),
    }
}

fn main() {
    let field = PBField {
        name: String::from("b"),
        tag: 2,
        field_type: PBType::String,
    };
    let msg = PBMessage {
        name: String::from("Test2"),
        fields: vec![field],
    };


    let msg = vec![0x12, 0x07, 0x74, 0x65, 0x73, 0x74, 0x69, 0x6e, 0x67];

    let (x, pp) = PBParseNext(msg,
                              ParserPosition {
                                  position: 0,
                                  tag: 0,
                                  wire_type: PBWireType::Reserved,
                                  cur_idx: 0,
                                  max_idx: 0,
                              });

    print!("Tag   = {}\n", x.tag);
    print!("Value = {}\n", String::from_utf8(x.value).unwrap());
}
