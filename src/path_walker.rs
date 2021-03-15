use std::{collections::HashMap, string};
use walkdir::WalkDir;
use std::time::{Duration, SystemTime};

pub struct FileData
{
    filename : String,
    modified : u64
}

impl FileData
{
    pub fn get_modified(&self) -> u64
    {
        self.modified
    }
}

fn strip_leading_separator(pathName: &String) -> String
{
    let ret: String;

    if pathName.len() > 0 && pathName.chars().next().unwrap() == std::path::MAIN_SEPARATOR
    {
        ret = pathName[1..].to_string();
    }
    else
    {
        ret = pathName.to_string();
    }

    ret
}

pub fn walk_And_gather_information(pathName: &String) -> Result<HashMap<String, FileData>, walkdir::Error>
{
    let mut ret = HashMap::new();
    let pathNameLen = pathName.len();
    for entryFile in WalkDir::new(pathName).into_iter()
                                                    .filter(|x| x.is_ok())
                                                    .map(|x| x.unwrap())
                                                    .filter(|x| x.path().is_file())
    {
        let mut modified = 0u64;
        if let Result::Ok(sysTime) = entryFile.metadata()?.modified()
        {
            if let Result::Ok(dur) = sysTime.duration_since(SystemTime::UNIX_EPOCH)
            {
                modified = dur.as_secs();
            }
        }
        
        if let Option::Some(path) = entryFile.path().as_os_str().to_str()
        {
            let relativePath = strip_leading_separator(&path[pathNameLen .. path.len()].to_string());
            println!("{} {}", relativePath, modified);
            ret.insert(relativePath.to_string(), FileData{
                filename: relativePath,
                modified: modified
            });
        }
    }
    Ok(ret)
}
