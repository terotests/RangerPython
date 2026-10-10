// SPDX-License-Identifier: MIT
//! The in-memory filesystem a script runs in. The host puts files in
//! before the run and reads the changed ones back after it.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub struct Fs {
    pub files: BTreeMap<String, String>,
    pub dirs: BTreeSet<String>,
    changed: Vec<String>,
    changed_set: BTreeSet<String>,
    host_files: BTreeSet<String>,
    pub cwd: String,
}

impl Fs {
    pub fn new() -> Fs {
        let mut fs = Fs { cwd: "/".into(), ..Default::default() };
        fs.dirs.insert("/".into());
        fs.dirs.insert("/tmp".into());
        fs
    }

    pub fn normalize(&self, path: &str) -> String {
        let p = if path.starts_with('/') {
            path.to_string()
        } else if self.cwd == "/" {
            format!("/{}", path)
        } else {
            format!("{}/{}", self.cwd, path)
        };
        let mut out: Vec<&str> = vec![];
        for part in p.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    out.pop();
                }
                x => out.push(x),
            }
        }
        format!("/{}", out.join("/"))
    }

    fn parent_of(abs: &str) -> String {
        match abs.rfind('/') {
            Some(i) if i > 0 => abs[..i].to_string(),
            _ => "/".into(),
        }
    }

    fn mark_changed(&mut self, abs: &str) {
        if self.changed_set.insert(abs.to_string()) {
            self.changed.push(abs.to_string());
        }
    }

    pub fn is_file(&self, path: &str) -> bool {
        self.files.contains_key(&self.normalize(path))
    }

    pub fn is_dir(&self, path: &str) -> bool {
        let abs = self.normalize(path);
        if abs == "/" || self.dirs.contains(&abs) {
            return true;
        }
        let prefix = format!("{}/", abs);
        self.files.range(prefix.clone()..).next().map(|(k, _)| k.starts_with(&prefix)).unwrap_or(false)
    }

    pub fn exists(&self, path: &str) -> bool {
        self.is_file(path) || self.is_dir(path)
    }

    pub fn read(&self, path: &str) -> Option<String> {
        self.files.get(&self.normalize(path)).cloned()
    }

    pub fn mkdirs(&mut self, path: &str) {
        let mut abs = self.normalize(path);
        while abs != "/" && !self.dirs.contains(&abs) {
            self.dirs.insert(abs.clone());
            abs = Self::parent_of(&abs);
        }
    }

    pub fn write(&mut self, path: &str, text: String) {
        let abs = self.normalize(path);
        self.mkdirs(&Self::parent_of(&abs));
        self.files.insert(abs.clone(), text);
        self.mark_changed(&abs);
    }

    pub fn remove(&mut self, path: &str) -> bool {
        let abs = self.normalize(path);
        if self.files.remove(&abs).is_some() {
            self.mark_changed(&abs);
            true
        } else {
            false
        }
    }

    pub fn remove_dir(&mut self, path: &str) -> bool {
        let abs = self.normalize(path);
        if !self.dirs.contains(&abs) || !self.list_dir(&abs).is_empty() {
            return false;
        }
        self.dirs.remove(&abs);
        true
    }

    pub fn list_dir(&self, path: &str) -> Vec<String> {
        let abs = self.normalize(path);
        let prefix = if abs == "/" { "/".to_string() } else { format!("{}/", abs) };
        let mut seen = BTreeSet::new();
        for a in self.files.keys().chain(self.dirs.iter()) {
            if a.starts_with(&prefix) && *a != abs {
                let rest = &a[prefix.len()..];
                let rest = rest.split('/').next().unwrap_or("");
                if !rest.is_empty() {
                    seen.insert(rest.to_string());
                }
            }
        }
        seen.into_iter().collect()
    }

    pub fn all_files(&self) -> Vec<String> {
        self.files.keys().cloned().collect()
    }

    /// a file the host provides; not reported as changed
    pub fn put(&mut self, path: &str, text: String) {
        let abs = self.normalize(path);
        self.mkdirs(&Self::parent_of(&abs));
        self.files.insert(abs.clone(), text);
        self.host_files.insert(abs);
    }

    pub fn changed_files(&self) -> Vec<String> {
        self.changed.iter().filter(|p| self.files.contains_key(*p)).cloned().collect()
    }

    pub fn removed_files(&self) -> Vec<String> {
        self.changed.iter().filter(|p| self.host_files.contains(*p) && !self.files.contains_key(*p)).cloned().collect()
    }

    pub fn clear_changes(&mut self) {
        self.changed.clear();
        self.changed_set.clear();
    }
}
