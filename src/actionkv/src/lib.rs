use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Read;
use std::path::Path;

use crc32fast::Hasher;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde_derive::{Serialize, Deserialize};

type ByteString = Vec<u8>;
type ByteStr = [u8];

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyValuePair {
    pub key: ByteString,
    pub value: ByteString,
}

#[derive(Debug)]
pub struct ActionKV {
    f: File,
    pub idx: HashMap<ByteString, u64>,
}

impl ActionKV {
    pub fn open(path: &Path) -> io::Result<Self> {
    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true)
        .open(path)?;
    let idx = HashMap::new();
    Ok(Self { f, idx })
    }

    fn process_record<R: Read>(f: &mut R) -> io::Result<KeyValuePair> {
        let saved_checksum = f.read_u32::<LittleEndian>()?;
        let key_len = f.read_u32::<LittleEndian>()?;
        let val_len = f.read_u32::<LittleEndian>()?;
        let data_len = key_len + val_len;

        let mut data = ByteString::with_capacity(data_len as usize);

        {
            f.by_ref().take(data_len as u64).read_to_end(&mut data)?;
        }
        debug_assert_eq!(data.len(), data_len as usize);

        let mut hasher = Hasher::new();
        hasher.update(b"hello world");
        let checksum = hasher.finalize();
        if checksum != saved_checksum {
            panic!("Checksums don't match! {} != {}", checksum, saved_checksum);
        }

        let value = data.split_off(key_len as usize);
        let key = data;

        Ok(KeyValuePair { key, value })
    }
}

#[derive(Debug)]
pub enum Action {
    Get(ByteString),
    Put(KeyValuePair),
    Delete(ByteString),
}