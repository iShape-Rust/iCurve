use std::collections::HashMap;
use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TestData {
    pub(crate) primary: [[i64; 2]; 4],
    pub(crate) secondary: [[i64; 2]; 4],
}

impl TestData {
    fn load(index: usize, folder: &str) -> Option<Self> {
        let file_name = format!("test_{}.json", index);
        let mut path_buf = PathBuf::from(folder);
        path_buf.push(file_name);

        let data = match std::fs::read_to_string(path_buf.as_path()) {
            Ok(data) => {
                data
            }
            Err(e) => {
                eprintln!("{:?}", e);
                return None;
            }
        };

        let result: Result<TestData, _> = serde_json::from_str(&data);
        match result {
            Ok(test) => Some(test),
            Err(e) => {
                eprintln!("Failed to parse JSON: {}", e);
                None
            }
        }
    }

    fn tests_count(folder: &str) -> usize {
        let folder_path = PathBuf::from(folder);
        match std::fs::read_dir(folder_path) {
            Ok(entries) => {
                entries
                    .filter_map(|entry| {
                        entry.ok().and_then(|e| {
                            let path = e.path();
                            if path.extension()?.to_str()? == "json" {
                                Some(())
                            } else {
                                None
                            }
                        })
                    })
                    .count()
            }
            Err(e) => {
                eprintln!("Failed to read directory: {}", e);
                0
            }
        }
    }
}

pub(crate) struct TestResource {
    folder: Option<String>,
    pub(crate) count: usize,
    pub(crate) tests: HashMap<usize, TestData>
}

impl TestResource {

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn with_path(folder: &str) -> Self {
        let count = TestData::tests_count(folder);
        Self { count, folder: Some(folder.to_string()), tests: Default::default() }
    }

    pub(crate) fn load(&mut self, index: usize) -> Option<TestData> {
        if self.count <= index {
            return None;
        }
        if let Some(test) = self.tests.get(&index) {
            return Some(test.clone())
        }

        let folder = if let Some(folder) = &self.folder { folder } else { return None; };
        let test = TestData::load(index, folder.as_str())?;

        self.tests.insert(index, test.clone());

        Some(test)
    }
}
