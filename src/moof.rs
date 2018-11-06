use bytes::{BytesMut, BufMut};

fn write_atom(parent: &mut BytesMut, id: &[u8; 4], atom: BytesMut) {
    parent.put_u32_be(atom.len() as u32 + 8_u32);
    parent.put_slice(&id[..]);
    parent.put_slice(atom.as_ref());
}

pub fn write_moof(parent: &mut BytesMut, sequence_number: u32, base_data_offset: u64, base_media_decode_time: u64, sample_sizes: Vec<u32>) {
    let mut buf = BytesMut::with_capacity(1024*1024);
    write_mfhd(&mut buf, sequence_number);
    write_traf(&mut buf, sample_sizes, base_data_offset, base_media_decode_time);

    write_atom(parent, b"moof", buf);
}

pub fn write_mfhd(parent: &mut BytesMut, sequence_number: u32) {
    let mut buf = BytesMut::with_capacity(10*1024*1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // 3 flags
    buf.put_u32_be(sequence_number);  // 4 sequence_number

    write_atom(parent, b"mfhd", buf);
}

pub fn write_traf(parent: &mut BytesMut, sample_sizes: Vec<u32>, base_data_offset: u64, base_media_decode_time: u64) {
    let mut buf = BytesMut::with_capacity(1024*1024);
//    buf.put_u32_be(30);  // 4 sample_number
//    buf.put_u32_be(29);  // 4 first_sample_index
    write_tfhd(&mut buf,base_data_offset, sample_sizes[0]);
    write_tfdt(&mut buf, base_media_decode_time);
    write_trun(&mut buf, sample_sizes);

    write_atom(parent, b"traf", buf);
}

pub fn write_tfhd(parent: &mut BytesMut, base_data_offset: u64, default_sample_size: u32) {
    let mut buf = BytesMut::with_capacity(1024*1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(57);  // 3 flags
    buf.put_u32_be(1); // 4 track_ID
    buf.put_u64_be(base_data_offset); // 4 base_data_offset
    //buf.put_u32_be(0); // 4 default_sample_description_index
    buf.put_u32_be(48000); // 4 default_sample_duration
    buf.put_u32_be(default_sample_size); // 4 default_sample_size
    buf.put_u32_be(16842752); // 4 default_sample_flags

    write_atom(parent, b"tfhd", buf);
}
pub fn write_tfdt(parent: &mut BytesMut, base_media_decode_time: u64) {
    let mut buf = BytesMut::with_capacity(1024*1024);
    buf.put_u8(1);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // 3 flags
    buf.put_u64_be(base_media_decode_time);  // 4 baseMediaDecodeTime

    write_atom(parent, b"tfdt", buf);
}
pub fn write_trun(parent: &mut BytesMut, sample_sizes: Vec<u32>) {
    let mut buf = BytesMut::with_capacity(1024*1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u16_be(517); // 3 flags

    let sample_count = sample_sizes.len() as u32;
    buf.put_u32_be(sample_count);  // 4 sample_count
    buf.put_u32_be(240);  // 4 data_offset
    buf.put_u32_be(33554432);  // 4 first_sample_flags

    // let sample_duration = [40000,39999,40000,39999,40000,40000,39999,40000,39999,40000,40000,39999,40000,39999,40000,40000,39999,40000,39999,40000,40000,39999,40000,39999,40000,40000,39999,40000,39999,40000];
    // let sample_size = [139090,616,27477,573,16247,560,16536,573,16213,649,16569,611,16137,575,20085,548,20076,623,26796,621,20147,577,19903,585,26552,618,25449,536,31778,548];
    for s in 0..sample_count as usize {
//        buf.put_u32_be(sample_duration[s] as u32);  // 4 sample_duration
        buf.put_u32_be(sample_sizes[s] as u32);  // 4 sample_size
//        buf.put_u32_be(0);  // 4 sample_info
//        buf.put_u32_be(0);  // 4 sample_count
    }

    write_atom(parent, b"trun", buf);
}

