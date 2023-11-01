use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, Read, SeekFrom};
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

    pub fn seek_to_end(&mut self) -> io::Result<u64> {
        self.f.seek(io::SeekFrom::End(0))
    }

    pub fn load(&mut self) -> io::Result<()> {
        let mut f = BufReader::new(&self.f);

        loop {
            let curr_pos = f.seek(io::SeekFrom::Current(0))?;

            let maybe_kv = ActionKV::process_record(&mut f);
            let kv = match maybe_kv {
                Ok(kv) => kv,
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::UnexpectedEof => break,
                        _ => return Err(e),
                    }
                }
            };
            self.idx.insert(kv.key, curr_pos);
        }
       Ok(())
    }

    pub fn get(&mut self, key: &ByteStr) -> io::Result<Option<ByteString>> {
        let pos = match self.idx.get(key) {
            Some(pos) => *pos,
            None => return Ok(None),
        };

        let kv = self.get_at(pos)?;
        Ok(Some(kv.value))
    }

    pub fn get_at(&mut self, pos: u64) -> io::Result<KeyValuePair> {
        let mut f = BufReader::new(&self.f);
        f.seek(SeekFrom::Start(pos))?;
        Ok(ActionKV::process_record(&mut f)?)
    }

    pub fn find(&mut self, key: &ByteStr) -> io::Result<Option<(u64, ByteString)>> {
        let mut f = BufReader::new(&self.f);
        let mut found: Option<(u64, ByteString)> = None;

        loop {
            let curr_pos = f.seek(io::SeekFrom::Current(0))?;

            let maybe_kv = ActionKV::process_record(&mut f);
            let kv = match maybe_kv {
                Ok(kv) => kv,
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::UnexpectedEof => break,
                        _ => return Err(e),
                    }
                }
            };

            if kv.key == key {
                found = Some((curr_pos, kv.value));
            }
        }
        Ok(found)
    }

    pub fn insert(&mut self, key: &ByteStr, val: &ByteStr) -> io::Result<()> {
        let pos = self.insert_but_ignore_index(key, val)?;
        self.idx.insert(key.to_vec(), pos);
        Ok(())
    }

    pub fn insert_but_ignore_index(&mut self, key: &ByteStr, val: &ByteStr) -> io::Result<u64> {
        let mut f = BufReader::new(&self.f);

        let key_len = key.len();
        let val_len = val.len();
        let mut tmp = ByteString::with_capacity(key_len + val_len);
        for b in key {
            tmp.push(*b);
        }

        for b in val {
            tmp.push(*b);
        }

        let mut hasher = Hasher::new();
        hasher.update(b"hello world");
        let checksum = hasher.finalize();

        let next_byte = SeekFrom::End(0);
        let pos = f.seek(SeekFrom::Current(0))?;
        f.seek(next_byte)?;

        f.write_u32::<LittleEndian>(checksum)?;
        f.write_u32::<LittleEndian>(key.len() as u32)?;
        f.write_u32::<LittleEndian>(val.len() as u32)?;
        f.write_all(&tmp)?;
        Ok(pos)
    }

    #[inline]
    pub fn update(
        &mut self,
        key: &ByteStr,
        value: &ByteStr,
    ) -> io::Result<()> {
        self.insert(key, value)
    }

    #[inline]
    pub fn delete(&mut self, key: &ByteStr) -> io::Result<()> {
        self.insert(key, b"")
    }
}

#[derive(Debug)]
pub enum Action {
    Get(ByteString),
    Put(KeyValuePair),
    Delete(ByteString),
}