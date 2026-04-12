use crate::rpi_image::Error;
use std::io::{Read, Seek};

#[derive(Debug, Clone, Copy)]
pub struct FatPartitionLayout {
  pub base: u64,
  pub length: u64,
}

const FAT32_WITH_CHS: u8 = 0xB;
const FAT32_WITH_LBA: u8 = 0xC;

impl FatPartitionLayout {
  pub fn new<R>(input_img: &mut R) -> Result<Self, Error>
  where
    R: Read + Seek,
  {
    let mbr = mbrman::MBR::read_from(input_img, 512)?;
    let Some((_, part)) = mbr
      .iter()
      .find(|(_, p)| p.sys == FAT32_WITH_CHS || p.sys == FAT32_WITH_LBA)
    else {
      return Err(Error::InvalidImage);
    };

    Ok(Self {
      base: part.starting_lba as u64 * mbr.sector_size as u64,
      length: part.sectors as u64 * mbr.sector_size as u64,
    })
  }
}
