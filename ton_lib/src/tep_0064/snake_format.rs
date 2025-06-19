#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnakeFormat(pub Vec<u8>);

impl SnakeFormat {
    pub fn new(data: Vec<u8>) -> Self { Self(data) }
}

impl From<SnakeFormat> for Vec<u8> {
    fn from(data: SnakeFormat) -> Self { data.0 }
}

impl From<Vec<u8>> for SnakeFormat {
    fn from(data: Vec<u8>) -> Self { SnakeFormat(data) }
}

impl AsRef<[u8]> for SnakeFormat {
    fn as_ref(&self) -> &[u8] { &self.0 }
}

// impl TLBType for SnakeFormat {
//     fn read_def(parser: &mut CellParser) -> Result<Self, TonLibError> {
//         // let mut cur_cell_parser = parser;
//         //
//         // let mut data_bits_len: u32 = cur_cell_parser.read_num(8)?;
//         // if data_bits_len % 8 != 0 {
//         //     return Err(TonLibError::TLBSnakeFormatUnsupportedBitsLen(data_bits_len));
//         // }
//         // let mut refs_left: u32 = TLBType::read(cur_cell_parser)?;
//         // let mut data = cur_cell_parser.read_bits(data_bits_len)?;
//         //
//         // while refs_left > 0 {
//         //     let ref_cell = cur_cell_parser.read_next_ref()?;
//         //     cur_cell_parser = &mut CellParser::new(ref_cell);
//         //     data_bits_len = cur_cell_parser.read_num(8)?;
//         //     if data_bits_len % 8 != 0 {
//         //         return Err(TonLibError::TLBSnakeFormatUnsupportedBitsLen(data_bits_len));
//         //     }
//         //     refs_left = cur_cell_parser.read_num(8)?;
//         //     data.extend(ref_cell.data);
//         //     refs_left -= 1;
//         // }
//         //
//
//
//         Ok(Self(data))
//     }
//
//     fn write_def(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> {
//         builder.write_bits(&self.0, SnakeFormat::BITS_LEN)?;
//         Ok(())
//     }
// }
