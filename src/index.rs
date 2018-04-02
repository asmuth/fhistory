use std::fs;
use std::path::{Path,PathBuf};
use regex::Regex;

const INDEX_FILENAME_PATTERN : &'static str =
    r"^fhistory-(?P<date>\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(\.\d+)?(([+-]\d\d:\d\d)|Z)?)-(?P<hash>[a-z0-9]+)$";

#[derive(Clone, Debug)]
pub struct IndexReference {
  pub date: String,
  pub hash: String,
}

pub struct IndexList {
  index_dir: PathBuf,
  index_files: Vec<IndexReference>,
}

pub struct IndexData {
}

impl IndexList {

  pub fn open(data_dir: &Path, index_dir: &Path) -> Result<IndexList, ::Error> {
    let index_dir : PathBuf = if index_dir.has_root() {
      index_dir.to_path_buf()
    } else {
      data_dir.join(index_dir)
    };

    let directory_listing = match fs::read_dir(&index_dir) {
      Ok(d) => d,
      Err(e) => return Err(e.to_string()),
    };

    let mut index_files = Vec::<IndexReference>::new();
    for entry in directory_listing {
      let entry = match entry {
        Ok(e) => e,
        Err(e) => return Err(e.to_string()),
      };

      let entry_fname = entry.file_name();
      let pattern = Regex::new(INDEX_FILENAME_PATTERN).unwrap();
      let pattern_match = match entry_fname.to_str().and_then(|x| pattern.captures(x)) {
        Some(m) => m,
        None => return Err(format!("invalid file in index directory: {:?}", entry_fname)),
      };

      index_files.push(IndexReference {
        date: pattern_match["date"].to_string(),
        hash: pattern_match["hash"].to_string()
      });
    }

    return Ok(IndexList {
      index_dir: index_dir,
      index_files: index_files,
    });
  }

  pub fn latest(self: &Self) -> Option<IndexReference> {
    return self.index_files.get(0).cloned();
  }

  pub fn list(self: &Self) -> &Vec<IndexReference> {
    return &self.index_files;
  }

}
