use std::io::prelude::*;
use std::io::Cursor;
use bytes::{IntoBuf, BytesMut, BufMut, Buf};

// h264_iso-iec_14496-10.pdf

pub struct IDR {
    pub samples : Vec<(NalUnitType,Vec<u8>)>
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum NalUnitType { //   Table 7-1 NAL unit type codes
    Unspecified = 0,                // Unspecified
    CodedSliceNonIdr = 1,           // Coded slice of a non-IDR picture
    CodedSliceDataPartitionA = 2,   // Coded slice data partition A
    CodedSliceDataPartitionB = 3,   // Coded slice data partition B
    CodedSliceDataPartitionC = 4,   // Coded slice data partition C
    CodedSliceIdr = 5,              // Coded slice of an IDR picture
    SEI = 6,                        // Supplemental enhancement information (SEI)
    SPS = 7,                        // Sequence parameter set
    PPS = 8,                        // Picture parameter set
    AUD = 9,                        // Access unit delimiter
    EndOfSequence = 10,             // End of sequence
    EndOfStream = 11,               // End of stream
    Filler = 12,                    // Filler data
    SpsExt = 13,                    // Sequence parameter set extension
                // 14..18           // Reserved
    NalUnitTypeCodedSliceAux = 19,  // Coded slice of an auxiliary coded picture without partitioning
                // 20..23           // Reserved
                // 24..31           // Unspecified
}

impl NalUnitType {
    pub fn from_u8(v: u8) -> Option<NalUnitType>  {
        match v {
            0  => Some(NalUnitType::Unspecified),
            1  => Some(NalUnitType::CodedSliceNonIdr),
            2  => Some(NalUnitType::CodedSliceDataPartitionA),
            3  => Some(NalUnitType::CodedSliceDataPartitionB),
            4  => Some(NalUnitType::CodedSliceDataPartitionC),
            5  => Some(NalUnitType::CodedSliceIdr),
            6  => Some(NalUnitType::SEI),
            7  => Some(NalUnitType::SPS),
            8  => Some(NalUnitType::PPS),
            9  => Some(NalUnitType::AUD),
            10 => Some(NalUnitType::EndOfSequence),
            11 => Some(NalUnitType::EndOfStream),
            12 => Some(NalUnitType::Filler),
            13 => Some(NalUnitType::SpsExt),
            19 => Some(NalUnitType::NalUnitTypeCodedSliceAux),
            _  => None
        }
    }
}


#[derive(Debug, Clone)]
pub struct NAL {
    pub start: usize,
    pub unit_type: NalUnitType,
    pub end: usize,
    pub data : Vec<u8>
}

fn get_forbidden_zero_bit(b: u8) -> u8 { (b & 0b10000000) >> 7 }
fn get_nal_ref_idc(b: u8) -> u8        { (b & 0b01100000) >> 5 }
fn get_nal_unit_type(b: u8) -> u8      { (b & 0b00011111) >> 0 }


//fn get_nal_ref_idc(b: u8) -> u8 {
////    let b = 0b10101010;
////    println!("{:b}", b & 0b10000000);
////    println!("{:b}", b & 0b10000000 << 1);
////    println!("{:b}", (b & 0b10000000) >> 7);
//    (b & 0b01110000) >> 4
//}

fn find_nal(buf: &Vec<u8>, offset: usize) -> Option<NAL> {
    let mut i = offset;

    loop { // start NAL     next_bits( 24 ) != 0x000001 && next_bits( 32 ) != 0x00000001
        if buf[i] == 0x00 && buf[i+1] == 0x00 && buf[i+2] == 0x01 { i+=3; break; }
        if buf[i] == 0x00 && buf[i+1] == 0x00 && buf[i+2] == 0x00 && buf[i+3] == 0x01 { i+=4; break; }
        i += 1;
        if i + 4 >= buf.len() { return None }
    }

    let mut nal = NAL{start: 0, unit_type: NalUnitType::Unspecified, end: 0, data: vec![]};
    nal.start = i;

    loop { // start NAL     next_bits( 24 ) != 0x000000 && next_bits( 24 ) != 0x000001
        if buf[i] == 0x00 && buf[i+1] == 0x00 && buf[i+2] == 0x01 { break; }
        if buf[i] == 0x00 && buf[i+1] == 0x00 && buf[i+2] == 0x00 && buf[i+3] == 0x01 { break; }
        i += 1;
        if i + 4 >= buf.len() { return None }
    }
    nal.end = i;

    Some(nal)
}

pub fn nal_from_data(buf: &Vec<u8>) -> Option<NAL> {
    let mut nal = NAL{start: 0, unit_type: NalUnitType::Unspecified, end: 0, data: vec![]};
    let b = buf[0];
    let nal_unit_type = get_nal_unit_type(b);
    nal.unit_type = NalUnitType::from_u8(nal_unit_type).unwrap();
    Some(nal)
}

pub fn get_nal(buf: &Vec<u8>, offset: usize) -> Option<NAL> {
    let mut i = offset;
    if i + 4 >= buf.len() { return None }

    let mut start: usize = 0;
    if buf[i] == 0x00 && buf[i+1] == 0x00 && buf[i+2] == 0x01 { start = i + 3; }
    if buf[i] == 0x00 && buf[i+1] == 0x00 && buf[i+2] == 0x00 && buf[i+3] == 0x01 { start = i + 4; }
    if start == 0 { println!("Can't find start"); return None; }

    let mut nal = NAL{start: 0, unit_type: NalUnitType::Unspecified, end: 0, data: vec![]};
    i = start;
    nal.start = i;

    let b = buf[start];
    // let nal_header = buf[i];
//    println!("b             {:b}    {:x}", b, b);

//    let forbidden_zero_bit = get_forbidden_zero_bit(b);
//    println!("forbidden_zero_bit {}", forbidden_zero_bit);
//
//    let nal_ref_idc = get_nal_ref_idc(b);
//    println!("nal_ref_idc {}", nal_ref_idc);
//
    let nal_unit_type = get_nal_unit_type(b);
//    println!("nal_unit_type {}", nal_unit_type);

    nal.unit_type = NalUnitType::from_u8(nal_unit_type).unwrap();


    loop { // start NAL     next_bits( 24 ) != 0x000000 && next_bits( 24 ) != 0x000001
        if buf[i] == 0x00 && buf[i+1] == 0x00 && buf[i+2] == 0x01 { break; }
        if buf[i] == 0x00 && buf[i+1] == 0x00 && buf[i+2] == 0x00 && buf[i+3] == 0x01 { break; }
        i += 1;
        if i + 4 >= buf.len() { return None }
    }
    nal.end = i;

    nal.data = Vec::from(&buf[nal.start .. nal.end]);

    Some(nal)
}

pub fn main_h264(path: &str) -> Result<(Vec<IDR>, NAL, NAL), ()> {
    let mut file = std::fs::File::open(path).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let bytes = contents.clone();

    let mut buf = Cursor::new(contents.clone());

    let mut offset = 0;
    let mut count = 0;

    let mut idrs = vec![];
    let mut idr = IDR{samples: vec![]};
    let mut first = true;
    let mut sps = NAL{start: 0, end: 0, data: vec![], unit_type: NalUnitType::SPS,};
    let mut pps = NAL{start: 0, end: 0, data: vec![], unit_type: NalUnitType::PPS};
    loop {
        let nal = get_nal(&contents, offset);
        if nal.is_none() { break }
        let nal = nal.unwrap();
        if first {
            if nal.unit_type == NalUnitType::SPS { sps = nal.clone(); }
            if nal.unit_type == NalUnitType::SPS { pps = nal.clone(); }
            first = false;
        }
        if nal.unit_type == NalUnitType::SPS {
            if idr.samples.len() > 0 {
                idrs.push(idr);
                idr = IDR{samples: vec![]};
            }
        }
        idr.samples.push((nal.unit_type.clone(), nal.data.clone()));
        println!("{}:     {:?}   size: {}", count, nal.unit_type, nal.end - nal.start);
        offset = nal.end;
        count += 1;
    }

    println!("count  {}", count);

//    let h = buf.get_u8();
//    println!("h: {:x?} ", h);
//
//
//
//    println!("{:x?} ", &bytes[0 .. 4]);
//    println!("{:x?} ", &bytes[4 .. 4 + 4]);


//    let mut cur = Cursor::new(contents);
//    let h = cur.read_u32::<BigEndian>()?;
//    println!("{:x?} ", h);
//    let h = cur.read_u32::<BigEndian>()?;

    // let pos = cur.position() as usize;

    Ok((idrs, sps, pps))
}
