use std::{
    fs::{remove_file, rename},
    path::PathBuf,
};

use tracing::debug;

pub struct TempFile {
    real_path: PathBuf,
    tmp_path: PathBuf,
}

impl TempFile {
    pub fn create(
        real_path: PathBuf,
        tmp_path: PathBuf,
    ) -> Result<TempFile, Box<dyn std::error::Error>> {
        debug!("using {:?} as tmp file", tmp_path);

        let temp_file = TempFile {
            real_path,
            tmp_path,
        };

        temp_file.cleanup_tmp_file()?;
        temp_file.cleanup_real_file()?;

        Ok(temp_file)
    }

    pub fn commit(self) -> Result<(), Box<dyn std::error::Error>> {
        debug!(
            "committing, renaming {:?} to {:?}",
            self.tmp_path, self.real_path
        );
        rename(self.tmp_path, self.real_path)?;
        Ok(())
    }

    pub fn abort(self) -> Result<(), Box<dyn std::error::Error>> {
        self.cleanup_tmp_file()?;

        Ok(())
    }

    fn cleanup_tmp_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.tmp_path.exists() {
            debug!("removing tmp file");
            remove_file(self.tmp_path.clone())?;
        }
        Ok(())
    }

    fn cleanup_real_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.real_path.exists() {
            debug!("removing real file");
            remove_file(self.real_path.clone())?;
        }
        Ok(())
    }
}
