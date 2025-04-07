use std::{
    fs::File,
    io::{Read, Write},
};

use serde::{Deserialize, Serialize};

pub trait Config: Sized + Serialize + for<'a> Deserialize<'a> {
    fn load<T>(s: T) -> Result<Self, String>
    where
        String: From<T>, T:Clone
    {
        let mut file_txt = String::new();
        let mut file = File::open(String::from(s.clone())).map_err(|_| format!("file {} not found",String::from(s.clone())))?;
        file.read_to_string(&mut file_txt)
            .map_err(|_| format!("couldn't read file {}",String::from(s.clone())))?;
        let result =
            toml::from_str(&file_txt).map_err(|_| format!("wrong json format in file {}",String::from(s.clone())))?;
        Ok(result)
    }
    fn save<T>(&self, s: T) -> Result<(), String>
    where
        String: From<T>, T:Clone
    {
        let file_txt =
            toml::to_string(self).map_err(|e| format!("{}",e.to_string()))?;
        let mut f =
            File::create(String::from(s.clone())).map_err(|_| format!("file {} could not be created",String::from(s.clone())))?;
        f.write_all(file_txt.as_bytes())
            .map_err(|_| format!("couldn't write to file {}",String::from(s.clone())))?;
        Ok(())
    }
}