use bytes::{BytesMut, BufMut};

pub struct MoovInfo {
    pub sps: Vec<u8>,
    pub pps: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub creation_time: u32,
    pub timescale: u32,
}

fn write_atom(parent: &mut BytesMut, id: &[u8; 4], atom: BytesMut) {
    parent.put_u32_be(atom.len() as u32 + 8_u32);
    parent.put_slice(&id[..]);
    parent.put_slice(atom.as_ref());
}

pub fn write_moov(parent: &mut BytesMut, moov_info: &MoovInfo) {
    let mut buf = BytesMut::with_capacity(1024*1024);
    write_mvhd(&mut buf, moov_info);
    write_trak(&mut buf, moov_info);
    write_mvex(&mut buf);
    // write_udta(&mut buf);
    write_atom(parent, b"moov", buf);
}

fn write_mvhd(parent: &mut BytesMut, moov_info: &MoovInfo) {
    let mut buf = BytesMut::with_capacity(1024*1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // 3 flags
    buf.put_u32_be(moov_info.creation_time);  // 4 creation_time
    buf.put_u32_be(0);  // 4 modification_time
    buf.put_u32_be(moov_info.timescale);  // 4 timescale
    buf.put_u32_be(0);  // 4 duration
    buf.put_u32_be(65536);  // 4 preferred rate
    buf.put_u16_le(1);  // 2 preferred volume
    buf.put_u16_be(0); buf.put_u32_be(0); buf.put_u32_be(0);  // 10 reserved
    {   // 36 matrix
        buf.put_u32_be(65536);
        buf.put_u32_be(0);
        buf.put_u32_be(0);
        buf.put_u32_be(0);
        buf.put_u32_be(65536);
        buf.put_u32_be(0);
        buf.put_u32_be(0);
        buf.put_u32_be(0);
        buf.put_u32_be(1073741824);
    }
    buf.put_u32_be(0);  // 4 Preview time
    buf.put_u32_be(0);  // 4 Preview duration
    buf.put_u32_be(0);  // 4 Poster time
    buf.put_u32_be(0);  // 4 Selection time
    buf.put_u32_be(0);  // 4 Selection duration
    buf.put_u32_be(0);  // 4 Current time
    buf.put_u32_be(2);  // 4 Next track ID

    write_atom(parent, b"mvhd", buf);
}

fn write_trak(parent: &mut BytesMut, moov_info: &MoovInfo) {
    let mut buf = BytesMut::with_capacity(1024);
    write_tkhd(&mut buf, moov_info);
    write_mdia(&mut buf, moov_info);
    write_atom(parent, b"trak", buf);
}
fn write_tkhd(parent: &mut BytesMut, moov_info: &MoovInfo) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(3);  // 3 flags
    buf.put_u32_be(moov_info.creation_time);  // 4 creation_time
    buf.put_u32_be(0);  // 4 modification_time
    buf.put_u32_be(1);  // 4 track id
    buf.put_u32_be(0);  // 4 reserved
    buf.put_u32_be(0);  // 4 duration
    buf.put_u64_be(0);  // 8 reserved
    buf.put_u16_be(0);  // 2 layer
    buf.put_u16_be(0);  // 2 Alternate group
    buf.put_u16_be(0);  // 2 Volume
    buf.put_u16_be(0);  // 2 Reserved
    {   // 36 Matrix structure
        buf.put_u32_be(65536);
        buf.put_u32_be(0);
        buf.put_u32_be(0);
        buf.put_u32_be(0);
        buf.put_u32_be(65536);
        buf.put_u32_be(0);
        buf.put_u32_be(0);
        buf.put_u32_be(0);
        buf.put_u32_be(1073741824);
    }
    buf.put_u32_be(125829120);  // 4 Track width
    buf.put_u32_be(70778880);  // 4 Track height

    write_atom(parent, b"tkhd", buf);
}

fn write_mdia(parent: &mut BytesMut, moov_info: &MoovInfo) {
    let mut buf = BytesMut::with_capacity(1024);
    write_mdhd(&mut buf, moov_info);
    write_hdlr(&mut buf, b"vide", "VideoHandler", b"\0\0\0\0");
    write_minf(&mut buf, moov_info);

    write_atom(parent, b"mdia", buf);
}

fn write_mdhd(parent: &mut BytesMut, moov_info: &MoovInfo) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // 3 flags
    buf.put_u32_be(0);  // 4 creation_time
    buf.put_u32_be(0);  // 4 modification_time
    buf.put_u32_be(moov_info.timescale);  // 4 timescale
    buf.put_u32_be(0);  // 4 duration
    buf.put_u16_be(21956);  // 2 language
    buf.put_u16_be(0);  // 2 quality

    write_atom(parent, b"mdhd", buf);
}

fn write_minf(parent: &mut BytesMut, moov_info: &MoovInfo) {
    let mut buf = BytesMut::with_capacity(1024);
    write_vmhd(&mut buf);
    write_dinf(&mut buf);
    write_stbl(&mut buf, moov_info);
    write_atom(parent, b"minf", buf);
}

fn write_dinf(parent: &mut BytesMut, ) {
    let mut buf = BytesMut::with_capacity(1024);
    write_dref(&mut buf);
    write_atom(parent, b"dinf", buf);
}

fn write_dref(parent: &mut BytesMut, ) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // 3 flags
    buf.put_u32_be(1); // 4 Component flags mask
    write_url(&mut buf);
    write_atom(parent, b"dref", buf);
}

fn write_url(parent: &mut BytesMut, ) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(1);  // 3 flags
    //buf.put_u8(0); // <counted string> end
    write_atom(parent, b"url ", buf);
}

fn write_vmhd(parent: &mut BytesMut, ) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(1);  // 3 flags
    buf.put_u16_be(0);  // 2 Graphics mode
    buf.put_u16_be(0);  // 2 Opcolor
    buf.put_u16_be(0);  // 2 Opcolor
    buf.put_u16_be(0);  // 2 Opcolor
    write_atom(parent, b"vmhd", buf);
}


fn write_stbl(parent: &mut BytesMut, moov_info: &MoovInfo) {
    let mut buf = BytesMut::with_capacity(1024);
    write_stsd(&mut buf, moov_info);
    write_stts(&mut buf);
    write_stsc(&mut buf);
    write_stsz(&mut buf);
    write_stco(&mut buf);

    write_atom(parent, b"stbl", buf);
}

fn write_stsd(parent: &mut BytesMut, moov_info: &MoovInfo) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // 3 flags
    buf.put_u32_be(1); // 4  Number of entries
    write_avc1(&mut buf, moov_info);

    write_atom(parent, b"stsd", buf);
}

fn write_avc1(parent: &mut BytesMut, moov_info: &MoovInfo) {
    let mut buf = BytesMut::with_capacity(1024);
//    buf.put_u16_be(0);  // 1 version
//    buf.put_u16_be(0);  // 1 revision
//    buf.put(&b"\0\0\0\0"[..]); // 4 vendor
//    buf.put_u32_be(0); // 4 temporal_quality
//    buf.put_u32_be(0); // 4 spatial_quality
//    buf.put_u16_be(0); // 2 width
//    buf.put_u16_be(0); // 2 height
//    buf.put_u16_be(0); // 2 horizontal_resolution
//    buf.put_u16_be(0); // 2 vertical_resolution
//
//    buf.put_u8(0);  // 1 version
//    buf.put_u8(0);  // 1 version

    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // reserved
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // reserved
    buf.put_u16_be(1); // data_reference_index
    buf.put_u16_be(0); // pre_defined
    buf.put_u16_be(0); // reserved
    buf.put_u32_be(0);
    buf.put_u32_be(0);
    buf.put_u32_be(0); // pre_defined
    buf.put_u16_be(moov_info.width); // 2 width
    buf.put_u16_be(moov_info.height); // 2 height
    buf.put_u32_be(moov_info.horizontal_resolution); // 4 horizontal_resolution
    buf.put_u32_be(moov_info.vertical_resolution); // 4 vertical_resolution
    buf.put_u32_be(0); // reserved
    buf.put_u16_be(1); // 2 frame_count
    buf.put_u8(0);
    buf.put(&[0, 0, 0, 0, // dailymotion/hls.js
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0][..]); // compressorname
    buf.put_u16_be(24); // 2 depth
    buf.put_u16_be(0xffff); // 2 color_table_id
    write_avcC(&mut buf, moov_info);

    write_atom(parent, b"avc1", buf);
}

#[allow(non_snake_case)]
fn write_avcC(parent: &mut BytesMut, moov_info: &MoovInfo) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(1);  // 1 version
    buf.put_u8(66);  // 1 profile
    buf.put_u8(0);  // 1 compatibility
    buf.put_u8(42);  // 1 level

    buf.put_u8(0xFF);  // 6 bits reserved (111111) + 2 bits nal size length - 1 (11)
    buf.put_u8(0xE1);  // 3 bits reserved (111) + 5 bits number of sps (00001)

    buf.put_u16_be(moov_info.sps.len() as u16);
    buf.put(moov_info.sps.as_slice()); // SPS

    buf.put_u8(1);  // 1 num pps
    buf.put_u16_be(moov_info.pps.len() as u16);
    buf.put(moov_info.pps.as_slice()); // pps

    write_atom(parent, b"avcC", buf);
}


fn write_stts(parent: &mut BytesMut, ) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // 3 flags
    buf.put_u32_be(0); // Number of entries
    // Time-to-sample table
    write_atom(parent, b"stts", buf);
}

fn write_stsc(parent: &mut BytesMut, ) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // 3 flags
    buf.put_u32_be(0); // Number of entries
    write_atom(parent, b"stsc", buf);
}

fn write_stsz(parent: &mut BytesMut, ) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // 3 flags
    buf.put_u32_be(0); // Sample size
    buf.put_u32_be(0); // Number of entries
    write_atom(parent, b"stsz", buf);
}

fn write_stco(parent: &mut BytesMut, ) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // 3 flags
    buf.put_u32_be(0); // Number of entries
    write_atom(parent, b"stco", buf);
}

fn write_mvex(parent: &mut BytesMut, ) {
    let mut buf = BytesMut::with_capacity(1024);
    write_trex(&mut buf);

    write_atom(parent, b"mvex", buf);
}

fn write_trex(parent: &mut BytesMut, ) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // 3 flags
    buf.put_u32_be(1); // track_ID
    buf.put_u32_be(1); // default_sample_description_index
    buf.put_u32_be(0); // default_sample_duration
    buf.put_u32_be(0); // default_sample_size
    buf.put_u32_be(0); // default_sample_flags
    write_atom(parent, b"trex", buf);
}

fn write_udta(parent: &mut BytesMut, ) {
    let mut buf = BytesMut::with_capacity(1024);
    write_meta(&mut buf);
    write_atom(parent, b"udta", buf);
}

fn write_meta(parent: &mut BytesMut, ) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // 3 flags
    write_hdlr(&mut buf, b"mdir", "", b"appl");
    write_ilst(&mut buf, &[0,0,0,37,169,116,111,111,0,0,0,29,100,97,116,97,0,0,0,1,0,0,0,0,76,97,118,102,53,55,46,56,51,46,49,48,48]);
    write_atom(parent, b"meta", buf);
}

fn write_hdlr(parent: &mut BytesMut, name: &[u8; 4], value: &'static str, manufacturer: &[u8; 4]) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // 3 flags
    buf.put_u32_be(0); // 4 Predefined
    buf.put(&name[..]); // 4 Component subtype
    buf.put(&manufacturer[..]); // 4 Component manufacturer
    buf.put_u32_be(0); // 4 Component flags
    buf.put_u32_be(0); // 4 Component flags mask
    buf.put(&value[..]); // <counted string> Component name
    buf.put_u8(0); // <counted string> end
    write_atom(parent, b"hdlr", buf);
}

fn write_ilst(parent: &mut BytesMut, arr: &[u8]) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put(&arr[..]); // <counted string> Component name
    write_atom(parent, b"ilst", buf);
}
