//! PCF process module.
//!
//! Reimplementation of [https://github.com/adafruit/Adafruit_CircuitPython_Bitmap_Font/blob/main/adafruit_bitmap_font/pcf.py](https://github.com/adafruit/Adafruit_CircuitPython_Bitmap_Font/blob/main/adafruit_bitmap_font/pcf.py)
//!
//! PCF fontforge page: [https://fontforge.org/docs/techref/pcf-format.html](https://fontforge.org/docs/techref/pcf-format.html)
//!
//! PCF format specification in kaitai format: [https://formats.kaitai.io/pcf_font/index.html](https://formats.kaitai.io/pcf_font/index.html)
//!
//! This lib only aims to read the glyphs in PCF fonts and interface with embedded-graphics.
//! Not all features are implemented.
//!
//! The properties table is dynamic, and may be implemented as an iterator.
//!
//! The metrics table stores per-glyph metric data.
//!
//! The bitmap table stores actual glyphs.
//!
//! PCF only supports 1 or 2 bytes encoding.
//!
//! TODO: `no_std` io::Seek and io::Read.

use core::fmt::Debug;
use num_enum::FromPrimitive;
#[cfg(feature = "std")]
use std::io;

use embedded_graphics::{
    image::{Image, ImageRaw},
    text::renderer::TextRenderer,
};

use crate::utils::*;

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Error {
    UnsupportedFormat,
    /// The data is not organized in an expected way, possibly currupted.
    CorruptedData,
    /// This error is raised when the code point is not found and there's no default char.
    NotFound,
    /// Something's wrong with IO operations.
    /// In some cases, currupted data also leads to IO error
    Io,
    Other,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        let _ = value;
        Self::Io
    }
}

/// There are at most 9 tables(one for each variant),
/// but only 4 of them are necessary to get the glyphs and misc data.
#[repr(u32)]
#[derive(Debug, Clone, Copy, FromPrimitive)]
enum TableType {
    /// Unknown type. This one is NOT in the specification!
    #[num_enum(default)]
    Unknown = 0,
    Properties = 1 << 0,
    Accelerators = 1 << 1,
    Metrics = 1 << 2,
    Bitmaps = 1 << 3,
    InkMetrics = 1 << 4,
    BdfEncodings = 1 << 5,
    Swidths = 1 << 6,
    GlyphNames = 1 << 7,
    BdfAccelerators = 1 << 8,
}

// Table formats
// /// Default table format
// const PCF_DEFAULT_FORMAT: u32 = 0x00000000;
// /// Unconfirmed flag
// const PCF_INKBOUNDS: u32 = 0x00000200;
/// Used by raw data format for [`Accelerator`] to indicate whether there to be
/// extra "ink bound" fields.
const PCF_ACCEL_W_INKBOUNDS: u32 = 0x00000100;
/// Used by metrics table to inicate whether the raw data is in compressed form.
const PCF_COMPRESSED_METRICS: u32 = 0x00000100;

// Table format extra flags
/// how each row in each glyph's bitmap is padded (format&3)
///  0=>bytes, 1=>shorts, 2=>ints
const PCF_GLYPH_PAD_MASK: u32 = 3 << 0;
/// If set then Most Sig Byte First
const PCF_BYTE_MASK: u32 = 1 << 2;
/// If set then Most Sig Bit First
const PCF_BIT_MASK: u32 = 1 << 3;
/// what the bits are stored in (bytes, shorts, ints) (format>>4)&3
/// 0=>bytes, 1=>shorts, 2=>ints
const PCF_SCAN_UNIT_MASK: u32 = 3 << 4;

/// Returns the length of each row in bytes.
const fn bytes_per_row(width: usize, bytes_align: usize) -> usize {
    let unit_align_bits = bytes_align * 8;
    // div floor
    let block_count = (width + unit_align_bits - 1) / unit_align_bits;
    block_count * bytes_align
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct TableTocEntry {
    format: u32,
    size: u32,
    offset: u32,
}

/// Uncompressed metrics data
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct MetricsEntry {
    left_side_bearing: i16,
    right_side_bearing: i16,
    character_width: i16,
    character_ascent: i16,
    character_descent: i16,
    character_attributes: u16,
}

impl MetricsEntry {
    /// Deserialize compressed data to create a [`MetricsEntry`]
    ///
    /// No boundary checking. The data length should be at least 5.
    fn new_from_compressed(data: &[u8]) -> Self {
        Self {
            left_side_bearing: data[0] as i16 - 0x80,
            right_side_bearing: data[1] as i16 - 0x80,
            character_width: data[2] as i16 - 0x80,
            character_ascent: data[3] as i16 - 0x80,
            character_descent: data[4] as i16 - 0x80,
            character_attributes: 0, // implied
        }
    }

    /// Deserialize uncompressed data to create a [`MetricsEntry`]
    ///
    /// No boundary checking. The data length should be at least 12.
    fn new_from_standard(data: &[u8]) -> Self {
        Self {
            left_side_bearing: i16_from_be_bytes_ref(&data[0..=2]),
            right_side_bearing: i16_from_be_bytes_ref(&data[2..=4]),
            character_width: i16_from_be_bytes_ref(&data[4..=6]),
            character_ascent: i16_from_be_bytes_ref(&data[6..=8]),
            character_descent: i16_from_be_bytes_ref(&data[8..=10]),
            character_attributes: u16_from_be_bytes_ref(&data[10..=12]),
        }
    }
}

/// Accelerator Tables
///
/// These data provide various bits of information about the font as a whole.
/// This data structure is used by two tables PCF_ACCELERATORS and PCF_BDF_ACCELERATORS.
/// The tables may either be in DEFAULT format or in PCF_ACCEL_W_INKBOUNDS (in which case
/// they will have some extra metrics data at the end.
///
/// The format field is ommitted as it just encoded font state.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct AcceleratorTable {
    no_overlap: u8,
    constant_metrics: u8,
    terminal_font: u8,
    constant_width: u8,
    ink_inside: u8,
    ink_metrics: u8,
    draw_direction: u8,
    padding: u8, // padding?
    font_ascent: i32,
    font_descent: i32,
    max_overlap: i32, // not used but takes some space
    minbounds: MetricsEntry,
    maxbounds: MetricsEntry,
    /// Exists if format is PCF_ACCEL_W_INKBOUNDS otherwise `should` be same with minbounds,
    ink_minbounds: Option<MetricsEntry>,
    /// Exists if format is PCF_ACCEL_W_INKBOUNDS otherwise `should` be same with maxbounds,
    ink_maxbounds: Option<MetricsEntry>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct EncodingTable {
    min_char_or_byte2: i16, /* As in XFontStruct */
    max_char_or_byte2: i16, /* As in XFontStruct */
    min_byte1: i16,         /* As in XFontStruct */
    max_byte1: i16,         /* As in XFontStruct */
    default_char: i16,      /* As in XFontStruct */
}

/// How glyphs rows are padded.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive)]
pub enum GlyphPaddingFormat {
    /// padded to one byte
    #[num_enum(default)]
    Byte,
    /// padded to 2 bytes
    Short,
    /// padded to 4 bytes
    Int,
}

#[derive(PartialEq)]
pub struct PcfFont<T> {
    data: T,
    /// Glyph count, informative. Original value is signed.
    glyph_count: u32,
    /// The number of pixels above the baseline of a typical ascender
    ascent: i32,
    /// The number of pixels below the baseline of a typical descender
    descent: i32,
    /// Whether metrics are compressed.
    metrics_compressed: bool,
    /// The maximum glyph size as a 4-tuple of: width, height, x_offset, y_offset
    bounding_box: (i16, i16, i16, i16),

    glyph_row_padding_format: GlyphPaddingFormat,
    // the 4 fields below actually only contains data of u8 size.
    min_char_or_byte2: u16, /* As in XFontStruct */
    max_char_or_byte2: u16, /* As in XFontStruct */
    min_byte1: u16,         /* As in XFontStruct */
    max_byte1: u16,         /* As in XFontStruct */
    default_char: u16,      /* As in XFontStruct */

    /// Use data here to get the glyph index of a code point.
    encoded_glyph_indices_location: u32,
    /// The absolute offset to bitmap offsets look up table in bitmap table
    ///
    /// Use glyph index against this to get the bitmap position.
    bitmap_position_lut_location: u32,
    /// The absolute offset to the first bitmap data in bitmap table
    ///
    /// Use bitmap position against this to get the glyph data.
    bitmap_data_location: u32,
    /// The absolute offset to the first metric entry in metrics table.
    ///
    /// Use glyph index against this to get the glyph metrics
    metrics_data_location: u32,
}

impl<T> PcfFont<T> {
    #[inline]
    pub fn bounding_box(&self) -> (i16, i16, i16, i16) {
        self.bounding_box
    }

    #[inline]
    pub fn glyph_count(&self) -> u32 {
        self.glyph_count
    }

    #[inline]
    pub fn ascent(&self) -> i32 {
        self.ascent
    }

    #[inline]
    pub fn desent(&self) -> i32 {
        self.descent
    }

    #[inline]
    pub fn row_padding_mode(&self) -> GlyphPaddingFormat {
        self.glyph_row_padding_format
    }

    #[inline]
    pub fn max_bytes_per_glyph(&self) -> usize {
        let width = self.bounding_box.0 as usize;
        let height = self.bounding_box.1 as usize;
        let row_bytes = bytes_per_row(width, 1);
        height * row_bytes
    }

    /// Override the default character.
    ///
    /// Whether this field is used depends on the implementation.
    #[inline]
    pub fn override_default_char(&mut self, value: u16) {
        self.default_char = value;
    }
}

impl<T> PcfFont<T>
where
    T: io::Read + io::Seek,
{
    /// Read raw glyph data of the given code_point, return `(length, width)`
    /// where `length` is the length of data written, the `width` is the glyph's width.
    /// Glyph rows are always padded to bytes.
    ///
    /// There might be arbitrary glyph sizes. Use the bounding box or [PcfFont::max_bytes_per_glyph
    /// to calculate the maximum required buffer size.
    ///
    /// In some cases the glyph will be empty, while it still needs space when displaying it.
    pub fn read_glyph_raw(
        &mut self,
        code_point: u16,
        buf: &mut [u8],
    ) -> Result<(usize, usize), Error> {
        let glyph_index = self.get_glyph_index(code_point)?;
        let bitmap_offset = self.get_glyph_bitmap_offset(glyph_index)?;
        let metrics = self.get_metrics(glyph_index)?;

        let glyph_width = (metrics.right_side_bearing - metrics.left_side_bearing) as usize;
        let glyph_height = (metrics.character_ascent + metrics.character_descent) as usize;
        let original_row_bytes = match self.glyph_row_padding_format {
            GlyphPaddingFormat::Byte => bytes_per_row(glyph_width, 1),
            GlyphPaddingFormat::Short => bytes_per_row(glyph_width, 2),
            GlyphPaddingFormat::Int => bytes_per_row(glyph_width, 4),
        };
        // convert all padding scheme to padding to bytes
        let standard_row_bytes = bytes_per_row(glyph_width, 1);
        self.data.seek(io::SeekFrom::Start(
            (self.bitmap_data_location + bitmap_offset) as u64,
        ))?;
        let skip_count = original_row_bytes - standard_row_bytes;
        // NOTE: this procedure is for MSBit-first glyphs
        for row in 0..glyph_height {
            let buf_start = row * standard_row_bytes;
            let buf_end = buf_start + standard_row_bytes;
            self.data.read_exact(&mut buf[buf_start..buf_end])?;
            // skip extra padding bytes
            self.data.seek_relative(skip_count as i64)?;
        }
        // the length of data written, the width of the bitmap
        let length = glyph_height * standard_row_bytes;
        Ok((length, glyph_width))
    }

    fn get_glyph_index(&mut self, code_point: u16) -> Result<u16, Error> {
        let enc1 = (code_point >> 8) & 0xFF;
        let enc2 = code_point & 0xFF;
        if !(self.min_byte1..=self.max_byte1).contains(&enc1)
            || !(self.min_char_or_byte2..=self.max_char_or_byte2).contains(&enc2)
        {
            return Err(Error::NotFound);
        }

        // for 1 or 2 bytes encoding, the procedure is the same.
        let indice_offset = (enc1 - self.min_byte1)
            * (self.max_char_or_byte2 - self.min_char_or_byte2 + 1)
            + (enc2 - self.min_char_or_byte2);
        // NOTE: each indice takes 2 bytes(u16)
        self.data.seek(io::SeekFrom::Start(
            (self.encoded_glyph_indices_location + (indice_offset as u32) * 2) as u64,
        ))?;
        let mut buffer: [u8; 2] = [0; 2];
        self.data.read_exact(&mut buffer[..])?;
        let glyph_index = u16::from_be_bytes(buffer);
        // 0xFFFF means there's no matching glyph
        if glyph_index == 0xFFFF {
            Err(Error::NotFound)
        } else {
            Ok(glyph_index)
        }
    }

    fn get_glyph_bitmap_offset(&mut self, glyph_index: u16) -> Result<u32, Error> {
        let mut buffer: [u8; 4] = [0; 4];
        // NOTE: each glyph location offset takes 4 bytes(u32)
        self.data.seek(io::SeekFrom::Start(
            (self.bitmap_position_lut_location + (glyph_index as u32) * 4) as u64,
        ))?;
        self.data.read_exact(&mut buffer)?;
        Ok(u32::from_be_bytes(buffer))
    }

    fn get_metrics(&mut self, glyph_index: u16) -> Result<MetricsEntry, Error> {
        if self.metrics_compressed {
            let cursor_offset = self.metrics_data_location + (glyph_index as u32) * 5;
            self.get_metrics_compressed(cursor_offset)
        } else {
            let cursor_offset = self.metrics_data_location + (glyph_index as u32) * 12;
            self.get_metrics_standard(cursor_offset)
        }
    }

    #[inline]
    fn get_metrics_compressed(&mut self, cursor_offset: u32) -> Result<MetricsEntry, Error> {
        self.data.seek(io::SeekFrom::Start(cursor_offset as u64))?;
        let mut buffer: [u8; 5] = [0; 5];
        self.data.read_exact(&mut buffer)?;
        Ok(MetricsEntry::new_from_compressed(&buffer))
    }

    #[inline]
    fn get_metrics_standard(&mut self, cursor_offset: u32) -> Result<MetricsEntry, Error> {
        self.data.seek(io::SeekFrom::Start(cursor_offset as u64))?;
        let mut buffer: [u8; 12] = [0; 12];
        self.data.read_exact(&mut buffer)?;
        Ok(MetricsEntry::new_from_standard(&buffer))
    }
}

impl<T> Debug for PcfFont<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PcfFont")
            .field("glyph_count", &self.glyph_count)
            .field("ascent", &self.ascent)
            .field("descent", &self.descent)
            .field("bounding_box", &self.bounding_box)
            .field("metrics_compressed", &self.metrics_compressed)
            .finish_non_exhaustive()
    }
}

/// Check and load PCF font using given IO buffer.
///
/// Use this to load the font, never try it manually.
pub fn load_pcf_font<T>(mut data: T) -> Result<PcfFont<T>, Error>
where
    T: io::Read + io::Seek,
{
    let mut buffer: [u8; 16] = [0; 16];
    data.rewind()?;

    // verify header
    data.read_exact(&mut buffer[..4])?;
    if buffer[..4] != [0x01, 0x66, 0x63, 0x70] {
        return Err(Error::UnsupportedFormat);
    }

    // read necessary tables(here only the table of content entries)
    let mut table_toc: [Option<TableTocEntry>; 5] = [None; 5];
    data.read_exact(&mut buffer[0..4])?;
    let table_count = u32_from_le_bytes_ref(&buffer[0..4]) as usize;
    for _ in 0..table_count {
        data.read_exact(&mut buffer[..16])?;
        let table_type = u32_from_le_bytes_ref(&buffer[0..4]);
        let table_toc_entry = TableTocEntry {
            format: u32_from_le_bytes_ref(&buffer[4..8]),
            size: u32_from_le_bytes_ref(&buffer[8..12]),
            offset: u32_from_le_bytes_ref(&buffer[12..16]),
        };
        match TableType::from_primitive(table_type) {
            TableType::Bitmaps => table_toc[0] = Some(table_toc_entry),
            TableType::Metrics => table_toc[1] = Some(table_toc_entry),
            TableType::BdfEncodings => table_toc[2] = Some(table_toc_entry),
            TableType::BdfAccelerators => table_toc[3] = Some(table_toc_entry),
            TableType::Accelerators => table_toc[4] = Some(table_toc_entry),
            _ => {}
        }
    }

    // Verify tables tocs
    if table_toc[3].is_none() {
        // copy Accelerators to BdfAccelerators, they serve the same purpose
        table_toc[3] = table_toc[4];
    }
    for i in table_toc[..4].iter() {
        if i.is_none() {
            return Err(Error::CorruptedData);
        }
        if i.unwrap().format & (PCF_BYTE_MASK | PCF_BIT_MASK) != (PCF_BYTE_MASK | PCF_BIT_MASK) {
            // NOTE: only support Most Significant Byte first by the moment
            // NOTE: current implmentation only supports reading Most-Significant-Bit-First glyph data.
            return Err(Error::UnsupportedFormat);
        }
    }

    // Check bitmap format
    /* format contains flags that indicate: */
    /* the byte order (format&4 == 4 => MSByte first)*/
    /* the bit order (format&8 == 4 => MSBit first) */
    /* how each row in each glyph's bitmap is padded (format&3) */
    /*  0=>bytes, 1=>shorts, 2=>ints */
    /* what the bits are stored in (bytes, shorts, ints) (format>>4)&3 */
    /*  0=>bytes, 1=>shorts, 2=>ints */
    // So 0xE means: MSByte first, MSBit first, glyph row padded to int(4 bytes)
    if table_toc[0].unwrap().format & PCF_SCAN_UNIT_MASK != 0 {
        // only support bits stored in bytes
        // having no idea of others though
        return Err(Error::UnsupportedFormat);
    }
    let glyph_row_padding_format = table_toc[0].unwrap().format & PCF_GLYPH_PAD_MASK;
    // TODO: is this check necessary?
    if glyph_row_padding_format == PCF_GLYPH_PAD_MASK {
        return Err(Error::CorruptedData);
    }
    let glyph_row_padding_format =
        GlyphPaddingFormat::from_primitive(glyph_row_padding_format as u8);

    // reconstruct the data

    // process Bitmaps table
    // not everything is used
    data.seek(io::SeekFrom::Start(table_toc[0].unwrap().offset as u64 + 4))?;
    data.read_exact(&mut buffer[0..4])?;
    let glyph_count = u32_from_be_bytes_ref(&buffer);
    data.seek(io::SeekFrom::Current(glyph_count as i64 * 4))?; // seek to bitmapSizes
    data.read_exact(&mut buffer[0..12])?;
    // let bitmap_size = u32_from_be_bytes_ref(&buffer[8..12]); // original i32, should be fine

    // process Metrics table
    // not everything is used
    data.seek(io::SeekFrom::Start(table_toc[1].unwrap().offset as u64))?;
    data.read_exact(&mut buffer[0..8])?;
    let metrics_compressed = table_toc[1].unwrap().format & PCF_COMPRESSED_METRICS > 0;
    let metrics_count = {
        if metrics_compressed {
            u16_from_be_bytes_ref(&buffer[4..6]) as u32
        } else {
            u32_from_be_bytes_ref(&buffer[4..8])
        }
    };
    if metrics_count != glyph_count {
        return Err(Error::CorruptedData);
    }

    // process Encoding table
    // not everything is used
    // skip format field
    data.seek(io::SeekFrom::Start(table_toc[2].unwrap().offset as u64 + 4))?;
    data.read_exact(&mut buffer[0..10])?;
    let min_char_or_byte2 = u16_from_be_bytes_ref(&buffer[0..2]);
    let max_char_or_byte2 = u16_from_be_bytes_ref(&buffer[2..4]);
    let min_byte1 = u16_from_be_bytes_ref(&buffer[4..6]);
    let max_byte1 = u16_from_be_bytes_ref(&buffer[6..8]);
    let default_char = u16_from_be_bytes_ref(&buffer[6..8]);

    // process Accelerators table
    // not everything is used
    // skip format, and some u8 meta data
    data.seek(io::SeekFrom::Start(
        table_toc[3].unwrap().offset as u64 + 4 + 8,
    ))?;
    data.read_exact(&mut buffer[0..8])?;
    let ascent = i32_from_be_bytes_ref(&buffer[0..4]);
    let descent = i32_from_be_bytes_ref(&buffer[4..8]);
    // skip `maxOverlap`
    data.seek_relative(4)?;
    // load ink bounds on demand
    let bounding_box = {
        if table_toc[3].unwrap().format & PCF_ACCEL_W_INKBOUNDS > 0 {
            // skip minbounds and maxbounds, use ink_minbounds and ink_maxbounds instead
            data.seek_relative(24)?;
        }
        data.read_exact(&mut buffer[0..12])?;
        let minbounds = MetricsEntry::new_from_standard(&buffer);
        data.read_exact(&mut buffer[0..12])?;
        let maxbounds = MetricsEntry::new_from_standard(&buffer);
        let width = maxbounds.right_side_bearing - minbounds.left_side_bearing;
        let height = maxbounds.character_ascent + maxbounds.character_descent;
        (
            width,
            height,
            minbounds.left_side_bearing,
            -maxbounds.character_descent,
        )
    };

    let bitmap_position_lut_location = table_toc[0].unwrap().offset + 4 + 4;
    let bitmap_data_location = bitmap_position_lut_location + (glyph_count + 4) * 4;
    let metrics_data_location =
        table_toc[1].unwrap().offset + 4 + if metrics_compressed { 2 } else { 4 };
    let encoded_glyph_indices_location = table_toc[2].unwrap().offset + 4 + 5 * 2;

    // println!(
    //     "Bitmap data location: {}/{}/{}/{}",
    //     bitmap_data_location, metrics_data_location, encoded_glyph_indices_location, table_toc[2].unwrap().offset
    // );

    Ok(PcfFont {
        data,
        glyph_count,
        ascent,
        descent,
        metrics_compressed,
        bounding_box,
        glyph_row_padding_format,
        min_char_or_byte2,
        max_char_or_byte2,
        min_byte1,
        max_byte1,
        default_char,
        encoded_glyph_indices_location,
        bitmap_position_lut_location,
        bitmap_data_location,
        metrics_data_location,
    })
}

#[cfg(test)]
mod test {
    use io::Cursor;

    use super::*;

    /// Big endian, glyph row padded to int(4 bytes)
    const FONT_VARIABLE: &[u8] =
        include_bytes!("../test-fonts/fusion-pixel-12px-proportional-zh_hans-pad_to_int.pcf");
    /// Mono font, big endian, glyph row padded to byte
    const FONT_MONO: &[u8] =
        include_bytes!("../test-fonts/fusion-pixel-12px-monospaced-zh_hans.pcf");

    #[test]
    #[cfg(feature = "std")]
    fn std_loading_pcf_fonts() {
        // TODO: give a good example
        let cursor = Cursor::new(FONT_VARIABLE);
        let _ = load_pcf_font(cursor).unwrap();
        let cursor = Cursor::new(FONT_MONO);
        let _ = load_pcf_font(cursor).unwrap();
    }

    #[test]
    #[cfg(feature = "std")]
    fn std_loading_glyphs() {
        let mut buffer: [u8; 50] = [0; 50];
        let cursor = Cursor::new(FONT_VARIABLE);
        let mut font = load_pcf_font(cursor).unwrap();
        let (length, width) = font.read_glyph_raw('聰' as u16, &mut buffer).unwrap();
        println!("data length: {length}, glyph width: {width}");
        if width == 0 {
            // in some cases the glyph is 'empty'
            return;
        }
        let row_bytes = bytes_per_row(width, 1);
        let height = length / row_bytes;
        for row in 0..height {
            let row_start = row * row_bytes;
            let row_end = row_start + row_bytes;
            for pixels in buffer[row_start..row_end].iter() {
                print!("{:>08b}", pixels)
            }
            println!("");
        }
    }
}
