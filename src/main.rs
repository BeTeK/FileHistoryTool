#[macro_use]
extern crate lazy_static;


mod file_journal;
mod path_walker;
use std::{collections::HashMap, hash, io::Read, iter::successors, mem, sync::mpsc::Sender, vec};
use std::vec::Vec;
use std::sync::mpsc;
use file_journal::FileJournal;
use sha2::Sha512;
use sha2::Digest;
use std::thread;
use std::fs::OpenOptions;
use std::path::Path;
use std::fs::File;
use std::time::{Duration, SystemTime};

fn filter_changed_files(fileJournal: &file_journal::FileJournal,
                        currentFiles: &HashMap<String, path_walker::FileData>) -> Vec<String>
{
     currentFiles.iter().map(|item| (item.0, item.1, fileJournal.get_filename(item.0)))
            .filter(|item| !item.2.is_some() || (item.2.is_some() && item.1.get_modified() > item.2.unwrap().get_modified()))
            .map(|item| item.0.to_string())
            .collect()
        
}

fn process_missing_files(mut fileJournal: &mut file_journal::FileJournal,
                         currentFiles: &HashMap<String, path_walker::FileData>) {
    let missing_files: Vec<String> = fileJournal.get_filenames()
        .iter()
        .map(|filename| filename.to_string())
        .filter(|filename| !currentFiles.contains_key(filename))
        .collect();
    for filename in missing_files
    {
        fileJournal.write_missing_file(&filename);
    }
}

struct HashResult
{
    hash : Vec<u8>,
    hashType : file_journal::HashType,
    filename : String,
    modified : u64
}

enum JournalWriteMessageEnum 
{
    Finished,
    Message(HashResult)
}

fn get_modified_time(fileObj: &File) -> u64
{
    let mut ret = 0u64;
    if let Result::Ok(metadata) = fileObj.metadata()
    {
        if let Result::Ok(sysTime) = metadata.modified()
        {
            if let Result::Ok(dur) = sysTime.duration_since(SystemTime::UNIX_EPOCH)
            {
                ret = dur.as_secs();
            }
        }
    }

    ret
}

fn hash_thread(rootPath: String, files: Vec<String>, tx: mpsc::Sender<JournalWriteMessageEnum>)
{
    for filename in files
    {
        let full_path = Path::new(&rootPath).join(&filename);
        let file = OpenOptions::new()
                            .read(true)
                            .open(full_path);

        if let Result::Ok(mut fileObj) = file 
        {
            let hash = hash_file(&mut fileObj);
            let modified = get_modified_time(&fileObj);

            if hash.len() > 0
            {
                let hashResult = HashResult
                {
                    filename: filename,
                    hashType: file_journal::HashType::Sha512,
                    hash: hash,
                    modified: modified
                };
                tx.send(JournalWriteMessageEnum::Message(hashResult));
            }
        }
        else if let Result::Err(er) = file
        {
            println!("{:?}", er.kind());
        }
    }

    tx.send(JournalWriteMessageEnum::Finished);
}

fn hash_file(fileObj: &mut std::fs::File) -> Vec<u8>
{
    const bufferSize: usize = 1;
    let mut buffer: [u8; bufferSize] = [0; bufferSize];
    let mut diggest = Sha512::new();

    let mut readBytes = bufferSize;
    let mut ret= Vec::new();
    let mut success = true;
    while  readBytes != 0
    {
        let res = fileObj.read(&mut buffer);
        if let Result::Ok(count) = res
        {
            readBytes = count;
            if count > 0
            {
                diggest.update(&buffer[0..count]);
            }
        }
        else
        {
            success = false;
            break;
        }
    }
    
    if success
    {
        let result_hash = diggest.finalize();
        ret = Vec::from(result_hash.as_slice());
    }

    ret
}

fn do_update_journal(root_path: String, files: Vec<String>, fileJournal: &mut file_journal::FileJournal)
{
    let threadCount = 1;
    let (tx, rx) = mpsc::channel();
    let threadRootPath = root_path.clone();
    thread::spawn(|| hash_thread(threadRootPath, files, tx));

    let mut finishedThreads = 0;
    loop
    {
        let message = rx.recv().unwrap();
        if let JournalWriteMessageEnum::Message(hashResult) = message 
        {
            fileJournal.write_update_or_add_file(&hashResult.filename, 
                                                hashResult.hashType, 
                                                hashResult.hash.as_slice(), 
                                                hashResult.modified);
        }
        else
        {
            finishedThreads += 1;
            if(finishedThreads == threadCount)
            {
                break;
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
    let rootPath = String::from("D:\\prog\\FileHistoryTool\\testPaths");
    let mut past_state = file_journal::FileJournal::load_journal(String::from("foobar.journal")).unwrap();
    let current_files = path_walker::walk_And_gather_information(&rootPath).unwrap();
    process_missing_files(&mut past_state, &current_files);
    let files_to_be_processed = filter_changed_files(&mut past_state, &current_files);
    do_update_journal(rootPath.clone(), files_to_be_processed, &mut past_state);
}
