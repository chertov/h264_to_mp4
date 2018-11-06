use std::io::prelude::*;
use std::io::Cursor;
use bytes::{IntoBuf, BytesMut, BufMut, Buf};


#[derive(Debug, Clone, PartialEq)]
enum AtomType { //   Table 7-1 NAL unit type codes
    FTYP = u8_slice_to_u32(b"ftyp") as isize,
    MOOV = u8_slice_to_u32(b"moov") as isize,
    MOOF = u8_slice_to_u32(b"moof") as isize,
    MDAT = u8_slice_to_u32(b"mdat") as isize,
    MFRA = u8_slice_to_u32(b"mfra") as isize,
}

const fn u8_slice_to_u32(id: &[u8; 4]) -> u32 {
    (((((( 0u32 | (id[0] as u32))
        << 8) | (id[1] as u32))
        << 8) | (id[2] as u32))
        << 8) | (id[3] as u32)
}

fn print_types() {
    println!("ftyp = 0x{:x} ", u8_slice_to_u32(b"ftyp") );
    println!("moov = 0x{:x} ", u8_slice_to_u32(b"moov") );
    println!("moof = 0x{:x} ", u8_slice_to_u32(b"moof") );
    println!("mdat = 0x{:x} ", u8_slice_to_u32(b"mdat") );
}

impl AtomType {
    fn from_u32(v: u32) -> Option<AtomType>  {
        if v == AtomType::FTYP as u32 { return Some(AtomType::FTYP); }
        if v == AtomType::MOOV as u32 { return Some(AtomType::MOOV); }
        if v == AtomType::MOOF as u32 { return Some(AtomType::MOOF); }
        if v == AtomType::MDAT as u32 { return Some(AtomType::MDAT); }
        if v == AtomType::MFRA as u32 { return Some(AtomType::MFRA); }

        panic!("Unknown type {}    str:{}", v, AtomType::u32_to_typestr(v));
        None
    }
    fn u32_to_typestr(v: u32) -> String  {
        let bytes: [u8; 4] = unsafe { std::mem::transmute(v.to_be()) };
        String::from_utf8(bytes.to_vec()).unwrap()
    }
}

pub struct Atom {
    start: usize,
    end: usize,
    typeid: AtomType,
    data: Vec<u8>,
}

pub fn read_atom(data: &mut Cursor<Vec<u8>>) -> Option<Atom> {
    let len = data.get_u32_be() as usize;
    let atom_typeid = data.get_u32_be();
    let len = len - 8;


    let mut bytes = vec![0u8; len];
    let readed = data.read(&mut bytes).unwrap();
    // println!("readed {} len {} ", readed, len);
    if readed != len {
        panic!("can't readed {} len {} ", readed, len);
    }

    let atom = Atom{start: 0, typeid: AtomType::from_u32(atom_typeid).unwrap(), end: 0, data: bytes};
    Some(atom)
}

pub struct Sample {
    data: Vec<u8>
}

pub fn read_sample(data: &mut Cursor<Vec<u8>>) -> Option<Sample> {
    let len = data.get_u32_be() as usize;
    let len = len;
    let mut bytes = vec![0u8; len];
    let readed = data.read(&mut bytes).unwrap();
    // println!("readed {} len {} ", readed, len);
    if readed != len {
        panic!("can't readed {} len {} ", readed, len);
    }

    let sample = Sample{data: bytes};
    Some(sample)
}

pub fn main_mp4_parser() -> std::io::Result<()> {
    let mut file = std::fs::File::open("output.mp4")?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    let mut buf = Cursor::new(contents.clone());

    let mut offset = 0;
    let mut count = 0;
    loop {
        if buf.position() as usize == contents.len() { break };
        let atom = read_atom(&mut buf);
        if atom.is_none() { break; }
        let atom= atom.unwrap();
        println!("{}   {:?}", count, atom.typeid);

        if atom.typeid == AtomType::MDAT {
            println!("{}     mdat size {:?}", count, atom.data.len());

            let mut mdat = Cursor::new(atom.data.clone());

            let mut offset = 0;
            let mut count = 0;
            let mut all_size = 0;
            loop {
                if mdat.position() as usize == atom.data.len() { break };
                let sample = read_sample(&mut mdat);
                if sample.is_none() { break; }
                let sample= sample.unwrap();
                let nal = crate::h264::nal_from_data(&sample.data).unwrap();
                println!("         sample size = {:?},   nal = {:?}", sample.data.len(), nal.unit_type);
                all_size += 4 + sample.data.len();
                count += 1;
            }
            println!("       mdat samples: count = {}, size = {:?}", count, all_size);
        }
        count += 1;
    };

    println!("count  {}", count);
    Ok(())
}
