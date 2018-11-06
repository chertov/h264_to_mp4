use std::io::prelude::*;
use bytes::{BytesMut, BufMut};

mod moov;
mod moof;
mod h264;
mod mp4_parser;

fn write_atom(parent: &mut BytesMut, id: &[u8; 4], atom: BytesMut) {
    parent.put_u32_be(atom.len() as u32 + 8_u32);
    parent.put_slice(&id[..]);
    parent.put_slice(atom.as_ref());
}

fn write_ftyp(parent: &mut BytesMut, ) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put(&b"isom"[..]);      // major_brand
    buf.put_u32_be(0x00000200_u32);          // minor_version
    buf.put(&b"isom"[..]);
    buf.put(&b"iso2"[..]);
    buf.put(&b"avc1"[..]);
    buf.put(&b"iso6"[..]);
    buf.put(&b"mp41"[..]);

    write_atom(parent, b"ftyp", buf);
}

fn write_mdat(parent: &mut BytesMut, data: Vec<u8>) {
    let mut buf = BytesMut::with_capacity(10*1024*1024);
    buf.put_slice(data.as_slice());
    write_atom(parent, b"mdat", buf);
}


fn write_samples(parent: &mut BytesMut, samples: & Vec<(h264::NalUnitType, Vec<u8>)>) -> (Vec<u32>, bytes::BytesMut) {

    if samples.len() < 4 { panic!("samples count is too small"); }

    let mut samples_sizes = vec![0u32];

    let mut all_size = 0;
    for (unit_type, sample) in samples {
        if unit_type != &h264::NalUnitType::CodedSliceNonIdr {
            samples_sizes[0] += sample.len() as u32 + 4;
        } else {
            samples_sizes.push(sample.len() as u32 + 4);
        }
        all_size += sample.len() + 4;
    }
    println!("   samples: {}    all_size: {}     samples_sizes {:?}", samples.len(), all_size, samples_sizes);

    let mut mdat_buf = BytesMut::with_capacity(all_size);
    for (unit_type, sample) in samples {
        mdat_buf.put_u32_be(sample.len() as u32);  // 4 sample_count
        mdat_buf.put_slice(sample.as_slice());
        // sample.len() + 4;
    }
    (samples_sizes, mdat_buf)
//
////
////
////        let samples_sizes = vec![23386,40,382,82,54,62,49,74,102,108,110,101,95,100,165,303,522,1074,5915,20497,41852,77201,74790,64662,53197,52811,41780,26423,17048,15035];
//    moof::write_moof(&mut buf, 1, samples_sizes);
//
//    write_atom(parent, b"ftyp", buf);
}

//fn main_mp4() {
//    let mut buf = BytesMut::with_capacity(10*1024*1024);
//    write_ftyp(&mut buf);
//    moov::write_moov(&mut buf);
//
//    let samples_sizes = vec![23386,40,382,82,54,62,49,74,102,108,110,101,95,100,165,303,522,1074,5915,20497,41852,77201,74790,64662,53197,52811,41780,26423,17048,15035];
//    moof::write_moof(&mut buf, 1, samples_sizes);
//
//    let mut file = std::fs::File::create("rust.mp4").unwrap();
//    file.write(buf.as_ref()).unwrap();
//}

fn main() {
    let mut buf = BytesMut::with_capacity(50*1024*1024);
    write_ftyp(&mut buf);
    moov::write_moov(&mut buf);



    // main_mp4();
    // mp4_parser::main_mp4_parser();
    let idrs = h264::main_h264("stream_chn0.h264").unwrap();

    println!("mdats: {} ", idrs.len());

    let mut seq = 0u32;
    for mdat in &idrs {
        let samples = &mdat.samples;
        let (samples_sizes, mdat_buf) = write_samples(&mut buf, samples);

        println!("buf size: {} ", buf.len());
        let base_data_offset = buf.len() as u64;
        let base_media_decode_time = 48000 * samples_sizes.len() as u64 * seq as u64;
        moof::write_moof(&mut buf, seq+1, base_data_offset, base_media_decode_time, samples_sizes);
        write_mdat(&mut buf, mdat_buf.to_vec());
        seq += 1;
    }

    let mut file = std::fs::File::create("rust.mp4").unwrap();
    file.write(buf.as_ref()).unwrap();
}
