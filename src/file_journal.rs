use std::{collections::HashMap, env::current_exe, fmt::LowerHex, hash, str::from_utf8, u64};
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use regex::Regex;

pub struct FileJournal
{
    file : File,
    current_state : HashMap<String, FileData>
}

pub enum HashType
{
    Sha512
}

impl FileJournal
{


    pub fn load_journal(filename: String) -> io::Result<FileJournal>
    {
        let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&filename)?;
        let bufReader = BufReader::new(&file);
        let mut currentState = HashMap::new();
        


        for line in bufReader.lines().into_iter().filter(|x| x.is_ok()).map(|x| x.unwrap()) 
        {
            if &line[0 .. 3] == "1 M" || &line[0 .. 3] == "1 A"
            {
                update_file(&mut currentState, &line);
            }
            else if &line[0 .. 3] == "1 D"
            {
                delete_file(&mut currentState, &line);
            }
            
            println!("{}", line)
        }

        Ok(FileJournal
        {
            file: file,
            current_state: currentState
        })
    }

    pub fn get_filename(&self, name: &String) -> Option<&FileData>
    {
        self.current_state.get(name)
    }

    pub fn get_filenames(&self) -> Vec<&String>
    {
        self.current_state.keys().collect()
    }

    pub fn write_missing_file(&mut self, filename: &String)
    {
        self.file.write(format!("1 D {}\n", filename).as_bytes());
    }

    fn format_hash(&self, hash: &[u8]) -> String
    {
        let conversion_table = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f'];
        let mut bytes = Vec::new();
        for val in hash 
        {
            let first_byte: usize = ((val & 0xf0) >> 4).into();
            let second_byte: usize = (val & 0xf).into();
            bytes.push(conversion_table[first_byte]);
            bytes.push(conversion_table[second_byte]);
        }

        std::str::from_utf8(&bytes).unwrap().to_string()
    }

    pub fn write_update_or_add_file(&mut self, 
                                    filename: &String, 
                                    hash_type: HashType,
                                    hash: &[u8],
                                    modified: u64)
    {
        let opName;
        if self.current_state.contains_key(filename)
        {
            opName = "M";
        }
        else
        {
            opName = "A";
        }
        let hashTxt = self.format_hash(hash);

        self.file.write(format!("1 {} sha512 {} {} {}\n", opName, hashTxt, modified, filename).as_bytes());
    }
}

fn delete_file(current_state: &mut HashMap<String, FileData>, line: &String)
{
}

fn update_file(current_state: &mut HashMap<String, FileData>, line: &String) 
{
    // let re = Regex::new(r"1 [M|A] sha512 (?P<hash>[0-9a-f]*) (?P<modified>[0-9]*) (?P<filename>.*)").unwrap();
    lazy_static! {
         static ref re: Regex = Regex::new("1 [M|A] sha512 (?P<hash>[0-9a-f]*) (?P<modified>[0-9]*) (?P<filename>.*)").unwrap();
    }

    let captures = re.captures(line).unwrap();
    let filename = &captures["filename"];
    let modifiedTxt = &captures["modified"];
    let modified = modifiedTxt.parse::<u64>().unwrap();
    let data = FileData
    {
        filename: filename.to_string(),
        modified: modified
    };

    current_state.insert(filename.to_string(), data);
}

pub struct FileData
{
    modified : u64,
    filename : String
}

impl FileData
{
    pub fn get_modified(&self) -> u64
    {
        self.modified
    }
}
