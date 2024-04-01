use std::io::ErrorKind;
use std::io::SeekFrom;
use std::path::Path;

use log::warn;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;

use crate::error::Error;

/// functions for dealing with data file records
///
/// A data file is a series of records. Each record has a header which is the length of the record, including the 4
/// bytes for the length. Then the record which as far as this is concerned is a string. No columns are handled.
/// Records are always appended to the end. Records are deleted by replacing with '#' characters.
/// A vacuum process will come along and compact the files by removing the deleted records.

/// Search a data_file for a particular record prefix and delete all of them
pub async fn find_and_delete(file_path: &Path, record_prefix: &str) -> Result<(), Error> {
    let file_result = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(file_path)
        .await;

    let mut file = match file_result {
        Ok(f) => f,
        Err(e) => {
            return Err(Error::BadFileName(
                file_path.to_str().unwrap().to_string(),
                e,
            ))
        }
    };

    let mut buffer: Vec<u8> = Vec::with_capacity(10024);
    loop {
        let record_len_result = file.read_u64().await;
        // if getting the record length doesn't work because we've hit an eof that's probably fine as this file should
        // end between records.
        if record_len_result
            .as_ref()
            .is_err_and(|e| ErrorKind::UnexpectedEof == e.kind())
        {
            break;
        } else if record_len_result.is_err() {
            return Err(record_len_result.unwrap_err().into());
        }

        let record_len = record_len_result.unwrap() as usize;

        let len = file.read(&mut buffer[0..record_len]).await?;
        // if the length read is not the same as the record length then something has got corrupted in this file.
        if len != record_len {
            return Err(Error::BadIndexRecord);
        }

        let record = String::from_utf8(Vec::from(&buffer[0..record_len]))?;
        if record.starts_with(record_prefix) {
            // we need to erase this record.
            // jump back to where the start of the record
            file.seek(SeekFrom::Current(-(record_len as i64 - 4)))
                .await?;
            // replace the record with blanks
            let mut blanking = "#".repeat(record_len - 5);
            blanking.push('\n');
            let mut written = file.write(blanking.as_bytes()).await?;
            if written != record_len - 4 {
                warn!("Incomplete write of index blanking!!!");
                // ??? try and write the difference?
                let mut new_blanking = "#".repeat(record_len - 5 - written);
                new_blanking.push('\n');
                written = file.write(new_blanking.as_bytes()).await?;

                // The second write didn't work either. Give up.
                if written != record_len - 4 - written {
                    warn!("Incomplete write a second time!!! This is bad, aborting!");
                    //attempt to get the position in the file
                    let pos_result = file.stream_position().await;
                    if pos_result.is_err() {
                        warn!("Could not get stream position either! Things are badly wrong with the file system");
                    }

                    return Err(Error::IncompleteWrite(
                        file_path.to_str().unwrap().to_string(),
                        pos_result.unwrap_or(0),
                    ));
                }
            }
        }
    }

    Ok(())
}

pub async fn append(file: &Path, record: &str) -> Result<(), Error> {
    fs::create_dir_all(file.parent().unwrap()).await?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
        .await?;
    let record_bytes = record.as_bytes();

    file.write_u32((record_bytes.len() + 5) as u32).await?; // 4 bytes for the length, +1 for the new line on the end.
    file.write_all(record_bytes).await?;
    file.write_u8(b'\n').await?;

    Ok(())
}

pub async fn read_all(file: &Path) -> Result<Vec<String>, Error> {
    if !file.exists() {
        return Ok(Vec::new());
    }

    let mut file = fs::OpenOptions::new().read(true).open(file).await?;

    let mut buffer: Vec<u8> = Vec::with_capacity(10024);
    let mut result = Vec::new();
    loop {
        let record_len_result = file.read_u64().await;
        // if getting the record length doesn't work because we've hit an eof that's probably fine as this file should
        // end between records.
        if record_len_result
            .as_ref()
            .is_err_and(|e| ErrorKind::UnexpectedEof == e.kind())
        {
            break;
        } else if record_len_result.is_err() {
            return Err(record_len_result.unwrap_err().into());
        }

        let record_len = record_len_result.unwrap() as usize;

        let len = file.read(&mut buffer[0..record_len]).await?;
        // if the length read is not the same as the record length then something has got corrupted in this file.
        if len != record_len {
            return Err(Error::BadIndexRecord);
        }

        let record = String::from_utf8(Vec::from(&buffer[0..record_len]))?;
        // ignore deleted records
        if !record.chars().all(|c| c == '#') {
            result.push(record);
        }
    }

    Ok(result)
}

// Go through a file and remove all the deleted records.
pub fn vacuum(_file: &Path) -> Result<(), Error> {
    todo!();
}
