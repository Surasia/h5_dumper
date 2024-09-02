use anyhow::{bail, Result};
use bitflags::bitflags;
use byteorder::{ReadBytesExt, LE};
use flate2::bufread::ZlibDecoder;
use std::io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom};
use thiserror::Error;

use crate::common::BufReaderExt;

#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("Incorrect module version! Should be either 23 or 27. Found: {0}")]
    InvalidModuleVersion(u32),
    #[error("Module magic doesn't match! Expected 'mohd' found: {0}")]
    InvalidModuleMagic(String),
    #[error("Tag size is zero! This should not happen.")]
    EmptyTag,
    #[error("Non-compressed single block tag found! This should not happen.")]
    NonCompressedSingleTag,
}

#[derive(Default, Debug)]
pub struct ModuleHeader {
    pub magic: String,
    pub version: u32,
    pub module_id: u64,
    pub item_count: u32,
    pub manifest_count: u32,
    pub resource_index: i32,
    pub strings_size: u32,
    pub resource_count: u32,
    pub block_count: u32,
    pub build_version: u64,
    pub checksum: u64,
}

impl ModuleHeader {
    pub fn read<R: BufRead + BufReaderExt>(&mut self, reader: &mut R) -> Result<()> {
        self.magic = reader.read_fixed_string(4)?;
        if self.magic != "mohd" {
            bail!(ModuleError::InvalidModuleMagic(self.magic.clone()))
        }
        self.version = reader.read_u32::<LE>()?;
        if self.version != 27 && self.version != 23 {
            bail!(ModuleError::InvalidModuleVersion(self.version))
        }
        self.module_id = reader.read_u64::<LE>()?;
        self.item_count = reader.read_u32::<LE>()?;
        self.manifest_count = reader.read_u32::<LE>()?;
        self.resource_index = reader.read_i32::<LE>()?;
        self.strings_size = reader.read_u32::<LE>()?;
        self.resource_count = reader.read_u32::<LE>()?;
        self.block_count = reader.read_u32::<LE>()?;
        self.build_version = reader.read_u64::<LE>()?;
        if self.version == 27 {
            self.checksum = reader.read_u64::<LE>()?;
        }
        Ok(())
    }
}

bitflags! {
    #[derive(Default, Debug)]
    pub struct FileFlags: u8 {
        const COMPRESSED = 1 << 0;
        const HAS_BLOCKS = 1 << 1;
        const RAW_FILE = 1 << 2;
    }
}

#[derive(Default, Debug)]
pub struct ModuleFileEntry {
    pub name_offset: u32,
    pub parent_file_index: i32,
    pub resource_count: u32,
    pub first_resource_index: i32,
    pub block_count: u32,
    pub first_block_index: i32,
    pub data_offset: u64,
    pub total_compressed_size: u32,
    pub total_uncompressed_size: u32,
    pub header_alignment: u8,
    pub tag_alignment: u8,
    pub resource_alignment: u8,
    pub flags: FileFlags,
    pub global_tag_id: i32,
    pub asset_id: i64,
    pub asset_checksum: i64,
    pub group_tag: String,
    pub uncompressed_header_size: u32,
    pub uncompressed_tag_size: u32,
    pub uncompressed_resource_size: u32,
    pub header_block_count: i16,
    pub tag_block_count: i16,
    pub resource_block_count: i16,
    pub padding: i16,
    pub name: String,
    pub data: Vec<u8>,
}

impl ModuleFileEntry {
    pub fn read<R: BufRead + BufReaderExt>(&mut self, reader: &mut R) -> Result<()> {
        self.name_offset = reader.read_u32::<LE>()?;
        self.parent_file_index = reader.read_i32::<LE>()?;
        self.resource_count = reader.read_u32::<LE>()?;
        self.first_resource_index = reader.read_i32::<LE>()?;
        self.block_count = reader.read_u32::<LE>()?;
        self.first_block_index = reader.read_i32::<LE>()?;
        self.data_offset = reader.read_u64::<LE>()?;
        self.total_compressed_size = reader.read_u32::<LE>()?;
        self.total_uncompressed_size = reader.read_u32::<LE>()?;
        self.header_alignment = reader.read_u8()?;
        self.tag_alignment = reader.read_u8()?;
        self.resource_alignment = reader.read_u8()?;
        self.flags = FileFlags::from_bits_truncate(reader.read_u8()?);
        self.global_tag_id = reader.read_i32::<LE>()?;
        self.asset_id = reader.read_i64::<LE>()?;
        self.asset_checksum = reader.read_i64::<LE>()?;
        self.group_tag = reader.read_fixed_string(4)?.chars().rev().collect();
        self.uncompressed_header_size = reader.read_u32::<LE>()?;
        self.uncompressed_tag_size = reader.read_u32::<LE>()?;
        self.uncompressed_resource_size = reader.read_u32::<LE>()?;
        self.header_block_count = reader.read_i16::<LE>()?;
        self.tag_block_count = reader.read_i16::<LE>()?;
        self.resource_block_count = reader.read_i16::<LE>()?;
        self.padding = reader.read_i16::<LE>()?;
        Ok(())
    }

    pub fn read_name<R: BufRead + BufReaderExt + Seek>(
        &mut self,
        reader: &mut R,
        file_name_offset: u32,
    ) -> Result<()> {
        reader.seek(SeekFrom::Start(
            (file_name_offset + self.name_offset) as u64,
        ))?;
        self.name = reader.read_cstring()?;
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct ModuleBlock {
    pub checksum: u64,
    pub compressed_offset: u32,
    pub compressed_size: u32,
    pub uncompressed_offset: u32,
    pub uncompressed_size: u32,
    pub compressed: bool,
    pub padding: i32,
}

impl ModuleBlock {
    pub fn read<R: BufRead + BufReaderExt + Seek>(
        &mut self,
        reader: &mut R,
        is_forge: bool,
    ) -> Result<()> {
        if is_forge {
            self.checksum = reader.read_u64::<LE>()?;
        }
        self.compressed_offset = reader.read_u32::<LE>()?;
        self.compressed_size = reader.read_u32::<LE>()?;
        self.uncompressed_offset = reader.read_u32::<LE>()?;
        self.uncompressed_size = reader.read_u32::<LE>()?;
        self.compressed = reader.read_u32::<LE>()? != 0;
        if is_forge {
            self.padding = reader.read_i32::<LE>()?;
        }
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct H5Module {
    pub header: ModuleHeader,
    pub files: Vec<ModuleFileEntry>,
    pub resource_indices: Vec<i32>,
    pub blocks: Vec<ModuleBlock>,
    pub data_offset: u64,
}

impl H5Module {
    pub fn read<R: BufRead + BufReaderExt + Seek>(&mut self, reader: &mut R) -> Result<()> {
        self.header.read(reader)?;
        self.files = (0..self.header.item_count)
            .map(|_| {
                let mut file = ModuleFileEntry::default();
                file.read(reader).unwrap();
                file
            })
            .collect();

        let name_offset = reader.stream_position()?;

        for file in &mut self.files {
            file.read_name(reader, name_offset as u32)?
        }

        self.resource_indices = (0..self.header.resource_count)
            .map(|_| reader.read_i32::<LE>().unwrap())
            .collect();

        self.blocks = (0..self.header.block_count)
            .map(|_| {
                let mut block = ModuleBlock::default();
                block.read(reader, self.header.version == 27).unwrap();
                block
            })
            .collect();

        self.data_offset = reader.stream_position()?;

        for id in 0..self.files.len() {
            self.read_tag(id as u32, reader)?;
        }
        Ok(())
    }

    pub fn read_tag<R: BufRead + Seek>(&mut self, index: u32, reader: &mut R) -> Result<()> {
        let file = &mut self.files[index as usize];
        if file.total_uncompressed_size == 0 {
            bail!(ModuleError::EmptyTag)
        }

        let block_offset = file.data_offset + self.data_offset;

        if file.flags.contains(FileFlags::HAS_BLOCKS) {
            let mut data_buffer = vec![0u8; file.total_uncompressed_size as usize];

            let blocks = &self.blocks[file.first_block_index as usize
                ..(file.first_block_index + file.block_count as i32) as usize];

            for block in blocks {
                let mut block_buffer = vec![0u8; block.compressed_size as usize];
                let offset = block_offset + block.compressed_offset as u64;
                reader.seek(SeekFrom::Start(offset))?;
                reader.read_exact(&mut block_buffer)?;

                let cursor = Cursor::new(&block_buffer);
                let buffer_reader = BufReader::new(cursor);
                let mut output_buffer = vec![0u8; block.uncompressed_size as usize];

                if block.compressed {
                    let mut decompressor = ZlibDecoder::new(buffer_reader);
                    decompressor.read_exact(&mut output_buffer)?;
                } else {
                    output_buffer.copy_from_slice(&block_buffer);
                }

                let dest_start = block.uncompressed_offset as usize;
                let dest_end = dest_start + block.uncompressed_size as usize;
                data_buffer[dest_start..dest_end].copy_from_slice(&output_buffer);
            }

            file.data = data_buffer;
        } else {
            let mut file_buffer = vec![0u8; file.total_compressed_size as usize];
            let offset = block_offset;
            reader.seek(SeekFrom::Start(offset))?;
            reader.read_exact(&mut file_buffer)?;

            if file.flags.contains(FileFlags::COMPRESSED) {
                let mut decompressed_buffer = vec![0u8; file.total_uncompressed_size as usize];
                let mut decompressor = ZlibDecoder::new(&file_buffer[..]);
                decompressor.read_exact(&mut decompressed_buffer)?;
                file.data = decompressed_buffer;
            } else {
                bail!(ModuleError::NonCompressedSingleTag)
            }
        }

        Ok(())
    }
}
