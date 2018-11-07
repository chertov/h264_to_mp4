use bytes::{BytesMut, BufMut};

pub struct SampleInfo{
    pub duration: u32,
    pub size: u32,
    pub flags: u32,
}

#[derive(Clone, PartialEq, Debug)] pub enum SampleLeading { UNKNOWN=0, LEADINGDEP=1, NOTLEADING=2, LEADINGNODEP=3 }
#[derive(Clone, PartialEq, Debug)] pub enum SampleDepends { UNKNOWN=0, DEPENDS=1, NOTDEPENDS=2, RESERVED=3 }
#[derive(Clone, PartialEq, Debug)] pub enum SampleDepended { UNKNOWN=0, NOTDISPOSABLE=1, DISPOSABLE=2, RESERVED=3 }
#[derive(Clone, PartialEq, Debug)] pub enum SampleRedundancy { UNKNOWN=0, REDUNDANT=1, NOTREDUNDANT=2, RESERVED=3 }

#[derive(Clone, PartialEq, Debug)]
pub struct SampleFlags {
    is_leading: SampleLeading,
    depends_on: SampleDepends,
    is_depended_on: SampleDepended,
    has_redundancy: SampleRedundancy,
    is_non_sync_sample: bool,
    degradation_priority: u16,
}
impl SampleFlags {
    pub fn parse(flags: u32) -> SampleFlags {
        //bit(4) reserved=0;
        //unsigned int(2) is_leading;
        //unsigned int(2) sample_depends_on;
        //unsigned int(2) sample_is_depended_on; unsigned int(2) sample_has_redundancy;
        //bit(3) sample_padding_value;
        //bit(1) sample_is_non_sync_sample;
        //unsigned int(16) sample_degradation_priority;

        let is_leading : u8                     = ((flags & 0b0000_1100_0000_0000_0000_0000_0000_0000_u32) >> 26) as u8;
        let sample_depends_on : u8              = ((flags & 0b0000_0011_0000_0000_0000_0000_0000_0000_u32) >> 24) as u8;
        let sample_is_depended_on : u8          = ((flags & 0b0000_0000_1100_0000_0000_0000_0000_0000_u32) >> 22) as u8;
        let sample_has_redundancy : u8          = ((flags & 0b0000_0000_0011_0000_0000_0000_0000_0000_u32) >> 20) as u8;
        let sample_is_non_sync_sample : bool    = ((flags & 0b0000_0000_0000_0001_0000_0000_0000_0000_u32) >> 16) == 1;
        let sample_degradation_priority : u16   = ((flags & 0b0000_0000_0000_0000_1111_1111_1111_1111_u32) >> 0) as u16;

        let mut flags = SampleFlags{
            is_leading: SampleLeading::UNKNOWN,
            depends_on: SampleDepends::UNKNOWN,
            is_depended_on: SampleDepended::UNKNOWN,
            has_redundancy: SampleRedundancy::UNKNOWN,
            is_non_sync_sample: false,
            degradation_priority: 0,
        };
        if is_leading == SampleLeading::UNKNOWN as u8 { flags.is_leading = SampleLeading::UNKNOWN; }
        else if is_leading == SampleLeading::LEADINGDEP as u8 { flags.is_leading = SampleLeading::LEADINGDEP; }
        else if is_leading == SampleLeading::NOTLEADING as u8 { flags.is_leading = SampleLeading::NOTLEADING; }
        else if is_leading == SampleLeading::LEADINGNODEP as u8 { flags.is_leading = SampleLeading::LEADINGNODEP; }
        else { unimplemented!(); }

        if sample_depends_on == SampleDepends::UNKNOWN as u8 { flags.depends_on = SampleDepends::UNKNOWN; }
        else if sample_depends_on == SampleDepends::DEPENDS as u8 { flags.depends_on = SampleDepends::DEPENDS; }
        else if sample_depends_on == SampleDepends::NOTDEPENDS as u8 { flags.depends_on = SampleDepends::NOTDEPENDS; }
        else { unimplemented!(); }

        if sample_is_depended_on == SampleDepended::UNKNOWN as u8 { flags.is_depended_on = SampleDepended::UNKNOWN; }
        else if sample_is_depended_on == SampleDepended::NOTDISPOSABLE as u8 { flags.is_depended_on = SampleDepended::NOTDISPOSABLE; }
        else if sample_is_depended_on == SampleDepended::DISPOSABLE as u8 { flags.is_depended_on = SampleDepended::DISPOSABLE; }
        else { unimplemented!(); }

        if sample_has_redundancy == SampleRedundancy::UNKNOWN as u8 { flags.has_redundancy = SampleRedundancy::UNKNOWN; }
        else if sample_has_redundancy == SampleRedundancy::REDUNDANT as u8 { flags.has_redundancy = SampleRedundancy::REDUNDANT; }
        else if sample_has_redundancy == SampleRedundancy::NOTREDUNDANT as u8 { flags.has_redundancy = SampleRedundancy::NOTREDUNDANT; }
        else { unimplemented!(); }

        flags.is_non_sync_sample = sample_is_non_sync_sample;
        flags.degradation_priority = sample_degradation_priority;

        flags
    }
    pub fn serialize(&self) -> u32 {
        let mut f = 0_u32;
        f = f | self.is_leading.clone() as u32;
        f = f << 2; f = f | self.depends_on.clone() as u32;
        f = f << 2; f = f | self.is_depended_on.clone() as u32;
        f = f << 2; f = f | self.has_redundancy.clone() as u32;
        f = f << 3;
        f = f << 1; f = f | self.is_non_sync_sample as u32;
        f = f << 16; f = f | self.degradation_priority as u32;
        f
    }
}

fn write_atom(parent: &mut BytesMut, id: &[u8; 4], atom: BytesMut) {
    parent.put_u32_be(atom.len() as u32 + 8_u32);
    parent.put_slice(&id[..]);
    parent.put_slice(atom.as_ref());
}

pub fn write_moof(parent: &mut BytesMut, sequence_number: u32, base_data_offset: u64, base_media_decode_time: u64, default_sample_duration: u32, samples_info: Vec<SampleInfo>) {
    let mut buf = BytesMut::with_capacity(2*1024);
    write_mfhd(&mut buf, sequence_number);
    let data_offset = buf.len();
    write_traf(&mut buf, default_sample_duration, samples_info, base_data_offset, base_media_decode_time, data_offset);
    // println!("moof size {}: ", buf.len());

    write_atom(parent, b"moof", buf);
}

pub fn write_mfhd(parent: &mut BytesMut, sequence_number: u32) {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put_u8(0);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // 3 flags
    buf.put_u32_be(sequence_number);  // 4 sequence_number

    write_atom(parent, b"mfhd", buf);
}

pub fn write_traf(parent: &mut BytesMut, default_sample_duration: u32, samples_info: Vec<SampleInfo>, base_data_offset: u64, base_media_decode_time: u64, data_offset: usize) {
    let mut buf = BytesMut::with_capacity(1024);
//    buf.put_u32_be(30);  // 4 sample_number
//    buf.put_u32_be(29);  // 4 first_sample_index
    write_tfhd(&mut buf,base_data_offset, default_sample_duration,samples_info[0].size);
    write_tfdt(&mut buf, base_media_decode_time);
    let data_offset = data_offset + buf.len();
    write_trun(&mut buf, samples_info, data_offset);

    write_atom(parent, b"traf", buf);
}

pub fn write_tfhd(parent: &mut BytesMut, base_data_offset: u64, default_sample_duration: u32, default_sample_size: u32) {
    let mut buf = BytesMut::with_capacity(1024*1024);
    buf.put_u8(0);  // 1 byte version

    let mut flags = 0x0_u32;
    let base_data_offset_present = true;
    let sample_description_index_present = false;
    let default_sample_duration_present = true;
    let default_sample_size_present = true;
    let default_sample_flags_present = true;
    let duration_is_empty = false;
    let default_base_is_moof = false;

    if base_data_offset_present         { flags = flags | 0x000001; } // base-data-offset-present
    if sample_description_index_present { flags = flags | 0x000002; } // sample-description-index-present
    if default_sample_duration_present  { flags = flags | 0x000008; } // default-sample-duration-present
    if default_sample_size_present      { flags = flags | 0x000010; } // default-sample-size-present
    if default_sample_flags_present     { flags = flags | 0x000020; } // default-sample-flags-present
    if duration_is_empty                { flags = flags | 0x010000; } // duration-is-empty
    if default_base_is_moof             { flags = flags | 0x020000; } // default-base-is-moof
    // buf.put_u8(0); buf.put_u8(0); buf.put_u8(0x39);  // 3 flags
    // println!("tfhd flags: 0x{:06x}        0x{:02x}: 0x{:02x}: 0x{:02x}", flags, (flags >> 16) as u8, (flags >> 8) as u8, (flags >> 0) as u8);
    buf.put_u8((flags >> 16) as u8); buf.put_u8((flags >> 8) as u8); buf.put_u8((flags >> 0) as u8); // 3 flags


    buf.put_u32_be(1); // 4 track_ID
    if base_data_offset_present { buf.put_u64_be(base_data_offset); }
    // if sample_description_index_present { buf.put_u32_be(0); } // 4 default_sample_description_index
    if default_sample_duration_present { buf.put_u32_be(default_sample_duration); }
    if default_sample_size_present { buf.put_u32_be(default_sample_size); }
    if default_sample_flags_present { buf.put_u32_be(16842752); }

    write_atom(parent, b"tfhd", buf);
}
pub fn write_tfdt(parent: &mut BytesMut, base_media_decode_time: u64) {
    let mut buf = BytesMut::with_capacity(1024*1024);
    buf.put_u8(1);  // 1 version
    buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);  // 3 flags
    buf.put_u64_be(base_media_decode_time);  // 4 baseMediaDecodeTime

    write_atom(parent, b"tfdt", buf);
}
pub fn write_trun(parent: &mut BytesMut, samples_info: Vec<SampleInfo>, data_offset: usize) {
    let mut buf = BytesMut::with_capacity(1024*1024);
    let version = 0u8;
    buf.put_u8(version);  // 1 version

    let data_offset_present = true;
    let first_sample_flags_present = true;
    let sample_duration_present = false;
    let sample_size_present = true;
    let sample_flags_present = false;
    let sample_composition_time_offsets_present = false;

    {
        let mut flags = 0x0_u32;
        if data_offset_present        { flags = flags | 0x000001; } // 0x000001 data-offset-present.
        if first_sample_flags_present { flags = flags | 0x000004; } // 0x000004 first-sample-flags-present
        if sample_duration_present    { flags = flags | 0x000100; } // 0x000100 sample-duration-present
        if sample_size_present        { flags = flags | 0x000200; } // 0x000200 sample-size-present
        if sample_flags_present       { flags = flags | 0x000400; } // 0x000400 sample-flags-present
        // if sample_composition_time_offsets_present { flags = flags | 0x000800; } // 0x000800 sample-composition-time-offsets-present

        // println!("trup flags: 0x{:06x}        0x{:02x}: 0x{:02x}: 0x{:02x}", flags, (flags >> 16) as u8, (flags >> 8) as u8, (flags >> 0) as u8);
        buf.put_u8((flags >> 16) as u8); buf.put_u8((flags >> 8) as u8); buf.put_u8((flags >> 0) as u8); // 3 flags

    }

    let sample_count = samples_info.len() as u32;
    buf.put_u32_be(sample_count);  // 4 sample_count

    {
        let mut data_offset = data_offset + buf.len();
        if data_offset_present          { data_offset += 4; }
        if first_sample_flags_present   { data_offset += 4; }
        if sample_duration_present      { data_offset += sample_count as usize * 4; }
        if sample_size_present          { data_offset += sample_count as usize * 4; }
        if sample_flags_present         { data_offset += sample_count as usize * 4; }
        data_offset += 4+4; // trun atom size + id
        data_offset += 4+4; // traf atom size + id
        data_offset += 4+4; // moof atom size + id
        data_offset += 4+4; // mdat atom size + id
        // println!("trun data_offset size {}: ", data_offset);
        if data_offset_present { buf.put_i32_be(data_offset as i32); } // 4 data_offset
    }

    if first_sample_flags_present { buf.put_u32_be(33554432); } // 4 first_sample_flags

    for sample_info in samples_info {
        if sample_duration_present { buf.put_u32_be(sample_info.duration); } // 4 sample_duration
        if sample_size_present     { buf.put_u32_be(sample_info.size); }    // 4 sample_size
        // if sample_flags_present    { buf.put_u32_be(sample_flags[s] as u32); }    // 4 sample_flags
        // if sample_composition_time_offsets_present {
        //     if version == 0 { buf.put_u32_be( sample_composition_time_offsets[s] as u32); }
        //     else { buf.put_i32_be( sample_composition_time_offsets[s] as i32); }
        // }
    }

    write_atom(parent, b"trun", buf);
}

