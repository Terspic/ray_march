use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FileObserver<'a> {
    files: Vec<&'a Path>,
    metadata: Vec<fs::Metadata>,
}

impl<'a> FileObserver<'a> {
    pub fn new<T: AsRef<Path>>(files: &'a [T]) -> io::Result<Self> {
        let mut metadata = Vec::with_capacity(files.len());
        for file in files {
            metadata.push(fs::metadata(file)?);
        }
        Ok(Self {
            files: files.iter().map(|f| f.as_ref()).collect(),
            metadata,
        })
    }

    pub fn modified(&mut self) -> Vec<&'a Path> {
        self.files
            .iter_mut()
            .enumerate()
            .filter_map(|(i, f)| {
                let metadata = fs::metadata(f.clone()).unwrap();
                let modified_time = metadata.modified().unwrap();

                if modified_time > self.metadata[i].modified().unwrap() {
                    self.metadata[i] = metadata;
                    return Some(f.clone());
                }

                None
            })
            .collect()
    }
}
