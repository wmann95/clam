//! Module for transforming various file types into a VecDataset usable by CHAODA

use std::{fs::{read_to_string, File}, path::Path};

use crate::VecDataset;

pub enum FileType<'a>{
    CSV{ path: &'a Path, hasHeaders: bool }
}

/// Trait that describes how to convert the given file type into a Vec<Vec<>> of itself.
pub trait ClamBake where Self: Sized{
    fn bake(file: FileType) -> Result<Vec<Vec<Self>>, String>;
}

impl ClamBake for f32{
    fn bake(file: FileType) -> Result<Vec<Vec<Self>>, String> {
        
        match file {
            FileType::CSV{ path, hasHeaders } => { 
                
                let mut reader = csv::ReaderBuilder::new()
                    .has_headers(hasHeaders)
                    .from_path(path)
                    .map_err(|e| e.to_string())?;
                
                let out: Result<Vec<Vec<f32>>, String> = 
                    reader.records()
                    .into_iter()
                    .map(|record|{
                        record.map_err(|e| e.to_string())?
                            .iter()
                            .map(|field_result|{
                                field_result.parse::<f32>().map_err(|e| e.to_string())
                            }).collect::<Result<Vec<f32>, String>>()
                    }).collect();
                out
            }
        }
        
    }
}


#[cfg(test)]
mod tests{
    #[test]
    fn clam_bake_works(){
        
    }
}