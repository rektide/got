use std::path::PathBuf;

/// Decorator that adds modification time information
pub struct DateIter<I> {
    inner: I,
    work_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct FileStatusWithDate {
    pub status: crate::types::FileStatus,
    pub modified_time: std::time::SystemTime,
    pub size: u64,
}

impl<I> DateIter<I> {
    pub fn new(inner: I, work_dir: PathBuf) -> Self {
        Self { inner, work_dir }
    }
}

impl<I> Iterator for DateIter<I>
where
    I: Iterator<Item = anyhow::Result<crate::types::FileStatus>>,
{
    type Item = anyhow::Result<FileStatusWithDate>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next()? {
            Ok(status) => {
                let full_path = self.work_dir.join(&status.path);

                let metadata = match std::fs::metadata(&full_path) {
                    Ok(m) => m,
                    Err(e) => return Some(Err(e.into())),
                };

                let modified_time = match metadata.modified() {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e.into())),
                };

                let size = metadata.len();

                Some(Ok(FileStatusWithDate {
                    status,
                    modified_time,
                    size,
                }))
            }
            Err(e) => Some(Err(e)),
        }
    }
}
