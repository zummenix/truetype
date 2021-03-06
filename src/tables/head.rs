
use types::Fixed;
use Error;
use Result;
use types::{BBox, LocationFormat};
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

/// A font header.
///
/// The 'head' table contains global information about the font.
#[derive(Debug, Default)]
pub struct HEAD {
    version: Fixed,
    font_revision: Fixed,
    check_sum_adjustment: u32,
    magic_number: u32,
    flags: u16,
    units_per_em: u16,
    created: i64,
    modified: i64,
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
    mac_style: u16,
    lowest_rec_ppem: u16,
    font_direction_hint: i16,
    index_to_loc_format: u16, // In TrueType Reference Manual this field marked
                              // as `i16` but we changed it to `u16`
                              // since it can hold only 0 or 1.
    glyph_data_format: i16,
}

impl HEAD {
    /// Returns `head` font table.
    ///
    /// Attempts to read `data` starting from `offset` position.
    ///
    /// # Errors
    /// Returns error if there is not enough data to read or version of
    /// the `head` font table is not supported.
    pub fn from_data(data: &[u8], offset: usize) -> Result<HEAD> {
        if offset >= data.len() {
            return Err(Error::Malformed);
        }

        let mut cursor = Cursor::new(&data[offset..]);
        let version = Fixed(try!(cursor.read_i32::<BigEndian>()));
        if version != Fixed(0x00010000) {
            return Err(Error::HEADVersionIsNotSupported);
        }

        let mut head = HEAD::default();
        head.version = version;
        head.font_revision = Fixed(try!(cursor.read_i32::<BigEndian>()));
        head.check_sum_adjustment = try!(cursor.read_u32::<BigEndian>());
        head.magic_number = try!(cursor.read_u32::<BigEndian>());
        head.flags = try!(cursor.read_u16::<BigEndian>());
        head.units_per_em = try!(cursor.read_u16::<BigEndian>());
        head.created = try!(cursor.read_i64::<BigEndian>());
        head.modified = try!(cursor.read_i64::<BigEndian>());
        head.x_min = try!(cursor.read_i16::<BigEndian>());
        head.y_min = try!(cursor.read_i16::<BigEndian>());
        head.x_max = try!(cursor.read_i16::<BigEndian>());
        head.y_max = try!(cursor.read_i16::<BigEndian>());
        head.mac_style = try!(cursor.read_u16::<BigEndian>());
        head.lowest_rec_ppem = try!(cursor.read_u16::<BigEndian>());
        head.font_direction_hint = try!(cursor.read_i16::<BigEndian>());
        head.index_to_loc_format = try!(cursor.read_u16::<BigEndian>());
        if head.index_to_loc_format > 1 {
            return Err(Error::UnknownLocationFormat);
        }
        head.glyph_data_format = try!(cursor.read_i16::<BigEndian>());

        Ok(head)
    }

    #[cfg(test)]
    fn bytes(&self) -> Vec<u8> {
        use byteorder::WriteBytesExt;

        let mut data = vec![];
        data.write_i32::<BigEndian>(self.version.0).unwrap();
        data.write_i32::<BigEndian>(self.font_revision.0).unwrap();
        data.write_u32::<BigEndian>(self.check_sum_adjustment).unwrap();
        data.write_u32::<BigEndian>(self.magic_number).unwrap();
        data.write_u16::<BigEndian>(self.flags).unwrap();
        data.write_u16::<BigEndian>(self.units_per_em).unwrap();
        data.write_i64::<BigEndian>(self.created).unwrap();
        data.write_i64::<BigEndian>(self.modified).unwrap();
        data.write_i16::<BigEndian>(self.x_min).unwrap();
        data.write_i16::<BigEndian>(self.y_min).unwrap();
        data.write_i16::<BigEndian>(self.x_max).unwrap();
        data.write_i16::<BigEndian>(self.y_max).unwrap();
        data.write_u16::<BigEndian>(self.mac_style).unwrap();
        data.write_u16::<BigEndian>(self.lowest_rec_ppem).unwrap();
        data.write_i16::<BigEndian>(self.font_direction_hint).unwrap();
        data.write_u16::<BigEndian>(self.index_to_loc_format).unwrap();
        data.write_i16::<BigEndian>(self.glyph_data_format).unwrap();
        data
    }

    /// Returns the number of units per em for the font.
    ///
    /// This value should be a power of 2. Its range is from 64 through 16384.
    pub fn units_per_em(&self) -> f32 {
        self.units_per_em as f32
    }

    /// Returns the bounding box around all possible characters.
    #[allow(dead_code)]
    pub fn bounding_box(&self) -> BBox {
        BBox {
            x0: self.x_min as i32,
            y0: self.y_min as i32,
            x1: self.x_max as i32,
            y1: self.y_max as i32
        }
    }

    /// Returns the type of offset format used in the index to loc ('loca') table.
    pub fn location_format(&self) -> LocationFormat {
        match self.index_to_loc_format {
            0 => LocationFormat::Short,
            1 => LocationFormat::Long,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Error::*;
    use expectest::prelude::*;

    const SIZE: usize = 4 * 4 + 2 * 2 + 8 * 2 + 2 * 9;

    #[test]
    fn smoke() {
        let data = ::utils::read_file("tests/Tuffy_Bold.ttf");
        let offset = ::utils::find_table_offset(&data, 0, b"head").unwrap().unwrap();

        let head = HEAD::from_data(&data, offset).unwrap();
        assert_eq!(head.bytes(), &data[offset..offset + SIZE]);

        let mut head = HEAD::default();
        expect!(HEAD::from_data(&head.bytes(), 0)).to(be_err().value(HEADVersionIsNotSupported));

        head.version = ::types::Fixed(0x00010000);
        head.index_to_loc_format = 2;
        expect!(HEAD::from_data(&head.bytes(), 0)).to(be_err().value(UnknownLocationFormat));

        expect!(HEAD::from_data(&data, data.len())).to(be_err().value(Malformed));
    }
}
