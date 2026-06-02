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

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Cursor;

  #[test]
  fn test_fat_partition_layout_new() {
    let mbr_data = vec![0u8; 1024];
    let mut mbr = mbrman::MBR::new_from(&mut Cursor::new(&mbr_data), 512, [0; 4]).unwrap();

    mbr[1] = mbrman::MBRPartitionEntry {
      boot: mbrman::BOOT_INACTIVE,
      starting_lba: 100,
      sectors: 200,
      sys: FAT32_WITH_LBA,
      first_chs: mbrman::CHS::empty(),
      last_chs: mbrman::CHS::empty(),
    };

    let mut mbr_cursor = Cursor::new(vec![0u8; 1024]);
    mbr.write_into(&mut mbr_cursor).unwrap();
    mbr_cursor.set_position(0);

    let layout = FatPartitionLayout::new(&mut mbr_cursor).unwrap();
    assert_eq!(layout.base, 100 * 512);
    assert_eq!(layout.length, 200 * 512);
  }

  #[test]
  fn test_fat_partition_layout_no_fat() {
    let mbr_data = vec![0u8; 1024];
    let mut mbr = mbrman::MBR::new_from(&mut Cursor::new(&mbr_data), 512, [0; 4]).unwrap();

    mbr[1] = mbrman::MBRPartitionEntry {
      boot: mbrman::BOOT_INACTIVE,
      starting_lba: 100,
      sectors: 200,
      sys: 0x83, // Linux
      first_chs: mbrman::CHS::empty(),
      last_chs: mbrman::CHS::empty(),
    };

    let mut mbr_cursor = Cursor::new(vec![0u8; 1024]);
    mbr.write_into(&mut mbr_cursor).unwrap();
    mbr_cursor.set_position(0);

    let result = FatPartitionLayout::new(&mut mbr_cursor);
    assert!(matches!(result, Err(Error::InvalidImage)));
  }
}
