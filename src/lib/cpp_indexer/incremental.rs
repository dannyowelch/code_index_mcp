use crate::lib::cpp_indexer::symbol_extractor::{SymbolExtractor, ExtractedSymbol};
use crate::lib::storage::models::file_metadata::FileMetadata;
use sha2::{Sha256, Digest};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::time::Instant;

#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub content_hash: String,
    pub metadata_hash: String,
    pub last_modified: u64,
    pub size: u64,
    pub dependencies: Vec<PathBuf>,
    pub dependents: Vec<PathBuf>,
    pub symbols_hash: String,
}

#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub hash: String,
    pub file_path: Option<PathBuf>,
    pub children: Vec<String>,
    pub is_leaf: bool,
    pub last_updated: u64,
}

#[derive(Debug)]
pub struct MerkleTree {
    nodes: HashMap<String, MerkleNode>,
    root_hash: Option<String>,
    file_to_hash: HashMap<PathBuf, String>,
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_hash: None,
            file_to_hash: HashMap::new(),
        }
    }

    pub fn add_file_node(&mut self, file_node: FileNode) -> Result<(), Box<dyn std::error::Error>> {
        let hash = self.compute_file_hash(&file_node)?;
        
        let merkle_node = MerkleNode {
            hash: hash.clone(),
            file_path: Some(file_node.path.clone()),
            children: Vec::new(),
            is_leaf: true,
            last_updated: file_node.last_modified,
        };
        
        self.nodes.insert(hash.clone(), merkle_node);
        self.file_to_hash.insert(file_node.path, hash);
        
        self.recompute_root()?;
        Ok(())
    }

    pub fn remove_file_node(&mut self, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(hash) = self.file_to_hash.remove(file_path) {
            self.nodes.remove(&hash);
            self.recompute_root()?;
        }
        Ok(())
    }

    fn compute_file_hash(&self, file_node: &FileNode) -> Result<String, Box<dyn std::error::Error>> {
        let mut hasher = Sha256::new();
        hasher.update(&file_node.content_hash);
        hasher.update(&file_node.metadata_hash);
        hasher.update(&file_node.symbols_hash);
        hasher.update(&file_node.last_modified.to_be_bytes());
        
        for dep in &file_node.dependencies {
            hasher.update(dep.to_string_lossy().as_bytes());
        }
        
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn recompute_root(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let leaf_hashes: Vec<String> = self.nodes
            .iter()
            .filter(|(_, node)| node.is_leaf)
            .map(|(hash, _)| hash.clone())
            .collect();
        
        if leaf_hashes.is_empty() {
            self.root_hash = None;
            return Ok(());
        }
        
        self.root_hash = Some(self.compute_tree_hash(&leaf_hashes)?);
        Ok(())
    }

    fn compute_tree_hash(&mut self, hashes: &[String]) -> Result<String, Box<dyn std::error::Error>> {
        if hashes.is_empty() {
            return Ok(String::new());
        }
        
        if hashes.len() == 1 {
            return Ok(hashes[0].clone());
        }
        
        let mut next_level = Vec::new();
        
        for chunk in hashes.chunks(2) {
            let combined_hash = if chunk.len() == 2 {
                self.combine_hashes(&chunk[0], &chunk[1])?
            } else {
                chunk[0].clone()
            };
            
            let merkle_node = MerkleNode {
                hash: combined_hash.clone(),
                file_path: None,
                children: chunk.iter().map(|h| h.clone()).collect(),
                is_leaf: false,
                last_updated: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            };
            
            self.nodes.insert(combined_hash.clone(), merkle_node);
            next_level.push(combined_hash);
        }
        
        self.compute_tree_hash(&next_level)
    }

    fn combine_hashes(&self, hash1: &str, hash2: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut hasher = Sha256::new();
        hasher.update(hash1.as_bytes());
        hasher.update(hash2.as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn get_root_hash(&self) -> Option<&String> {
        self.root_hash.as_ref()
    }

    pub fn has_changed(&self, other_root_hash: &str) -> bool {
        match &self.root_hash {
            Some(root) => root != other_root_hash,
            None => !other_root_hash.is_empty(),
        }
    }

    pub fn get_changed_files(&self, other: &MerkleTree) -> Vec<PathBuf> {
        let mut changed_files = Vec::new();
        
        for (path, hash) in &self.file_to_hash {
            match other.file_to_hash.get(path) {
                Some(other_hash) if hash != other_hash => {
                    changed_files.push(path.clone());
                }
                None => {
                    changed_files.push(path.clone());
                }
                _ => {}
            }
        }
        
        changed_files
    }
}

pub struct IncrementalIndexer {
    symbol_extractor: SymbolExtractor,
    current_tree: MerkleTree,
    file_cache: HashMap<PathBuf, FileNode>,
    dependency_graph: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl IncrementalIndexer {
    pub fn new(compile_flags: Option<Vec<String>>) -> Result<Self, Box<dyn std::error::Error>> {
        let symbol_extractor = SymbolExtractor::new(compile_flags)?;
        
        Ok(Self {
            symbol_extractor,
            current_tree: MerkleTree::new(),
            file_cache: HashMap::new(),
            dependency_graph: HashMap::new(),
        })
    }

    pub async fn index_file(&mut self, file_path: &Path) -> Result<IncrementalResult, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        let file_metadata = self.get_file_metadata(file_path).await?;
        let content_hash = self.compute_content_hash(file_path).await?;
        
        let needs_reindex = if let Some(cached_node) = self.file_cache.get(file_path) {
            cached_node.content_hash != content_hash || 
            cached_node.last_modified != file_metadata.last_modified.timestamp() as u64
        } else {
            true
        };
        
        if !needs_reindex {
            return Ok(IncrementalResult {
                file_path: file_path.to_path_buf(),
                action: IndexAction::Skipped,
                affected_files: Vec::new(),
                symbols_extracted: 0,
                processing_time_ms: start_time.elapsed().as_millis() as u32,
            });
        }
        
        let extraction_result = self.symbol_extractor.extract_symbols(file_path).await?;
        let symbols_hash = self.compute_symbols_hash(&extraction_result.symbols)?;
        
        let dependencies = self.extract_file_dependencies(&extraction_result.includes).await?;
        let file_node = FileNode {
            path: file_path.to_path_buf(),
            content_hash,
            metadata_hash: self.compute_metadata_hash(&file_metadata)?,
            last_modified: file_metadata.last_modified.timestamp() as u64,
            size: file_metadata.size_bytes,
            dependencies: dependencies.clone(),
            dependents: Vec::new(),
            symbols_hash,
        };
        
        self.update_dependency_graph(file_path, &dependencies)?;
        let affected_files = self.get_affected_files(file_path)?;
        
        self.file_cache.insert(file_path.to_path_buf(), file_node.clone());
        self.current_tree.add_file_node(file_node)?;
        
        let processing_time = start_time.elapsed();
        
        Ok(IncrementalResult {
            file_path: file_path.to_path_buf(),
            action: IndexAction::Indexed,
            affected_files,
            symbols_extracted: extraction_result.symbols.len(),
            processing_time_ms: processing_time.as_millis() as u32,
        })
    }

    pub async fn remove_file(&mut self, file_path: &Path) -> Result<IncrementalResult, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        let affected_files = self.get_affected_files(file_path)?;
        
        self.file_cache.remove(file_path);
        self.current_tree.remove_file_node(file_path)?;
        self.dependency_graph.remove(file_path);
        
        for (_, deps) in self.dependency_graph.iter_mut() {
            deps.remove(file_path);
        }
        
        let processing_time = start_time.elapsed();
        
        Ok(IncrementalResult {
            file_path: file_path.to_path_buf(),
            action: IndexAction::Removed,
            affected_files,
            symbols_extracted: 0,
            processing_time_ms: processing_time.as_millis() as u32,
        })
    }

    pub async fn update_directory(&mut self, directory_path: &Path) -> Result<Vec<IncrementalResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        let cpp_extensions = [".cpp", ".cxx", ".cc", ".c", ".hpp", ".hxx", ".h"];
        
        let mut entries = fs::read_dir(directory_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if cpp_extensions.iter().any(|&ext| extension == &ext[1..]) {
                        let result = self.index_file(&path).await?;
                        results.push(result);
                    }
                }
            } else if path.is_dir() {
                let sub_results = Box::pin(self.update_directory(&path)).await?;
                results.extend(sub_results);
            }
        }
        
        Ok(results)
    }

    pub fn get_index_status(&self) -> IndexStatus {
        let total_files = self.file_cache.len();
        let total_dependencies = self.dependency_graph.values().map(|deps| deps.len()).sum();
        
        let file_types = self.file_cache
            .keys()
            .filter_map(|path| path.extension())
            .fold(HashMap::new(), |mut acc, ext| {
                *acc.entry(ext.to_string_lossy().to_string()).or_insert(0) += 1;
                acc
            });
        
        IndexStatus {
            total_files,
            total_dependencies,
            file_types,
            merkle_root: self.current_tree.get_root_hash().cloned(),
            last_updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        }
    }

    async fn get_file_metadata(&self, file_path: &Path) -> Result<FileMetadata, Box<dyn std::error::Error>> {
        let metadata = fs::metadata(file_path).await?;
        let last_modified = metadata.modified()?.into();
        
        Ok(FileMetadata {
            id: Some(0),
            index_id: uuid::Uuid::new_v4(),
            file_path: file_path.to_string_lossy().to_string(),
            file_hash: String::new(),
            last_modified,
            size_bytes: metadata.len(),
            symbol_count: 0,
            indexed_at: chrono::Utc::now(),
        })
    }

    async fn compute_content_hash(&self, file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let content = fs::read(file_path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn compute_symbols_hash(&self, symbols: &[ExtractedSymbol]) -> Result<String, Box<dyn std::error::Error>> {
        let mut hasher = Sha256::new();
        
        for symbol in symbols {
            hasher.update(&symbol.name);
            hasher.update(&format!("{:?}", symbol.symbol_type));
            hasher.update(&symbol.fully_qualified_name);
            hasher.update(&symbol.signature.as_ref().unwrap_or(&String::new()));
        }
        
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn compute_metadata_hash(&self, metadata: &FileMetadata) -> Result<String, Box<dyn std::error::Error>> {
        let mut hasher = Sha256::new();
        hasher.update(&metadata.size_bytes.to_be_bytes());
        hasher.update(&metadata.last_modified.timestamp().to_be_bytes());
        hasher.update(&metadata.file_path);
        Ok(format!("{:x}", hasher.finalize()))
    }

    async fn extract_file_dependencies(&self, includes: &[String]) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut dependencies = Vec::new();
        
        for include in includes {
            if include.ends_with(".h") || include.ends_with(".hpp") || include.ends_with(".hxx") {
                dependencies.push(PathBuf::from(include));
            }
        }
        
        Ok(dependencies)
    }

    fn update_dependency_graph(&mut self, file_path: &Path, dependencies: &[PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
        let deps_set: HashSet<PathBuf> = dependencies.iter().cloned().collect();
        self.dependency_graph.insert(file_path.to_path_buf(), deps_set);
        
        for dep in dependencies {
            if let Some(cached_node) = self.file_cache.get_mut(dep) {
                if !cached_node.dependents.contains(&file_path.to_path_buf()) {
                    cached_node.dependents.push(file_path.to_path_buf());
                }
            }
        }
        
        Ok(())
    }

    fn get_affected_files(&self, file_path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut affected = Vec::new();
        let mut visited = HashSet::new();
        
        self.collect_dependents_recursive(file_path, &mut affected, &mut visited);
        
        Ok(affected)
    }

    fn collect_dependents_recursive(
        &self,
        file_path: &Path,
        affected: &mut Vec<PathBuf>,
        visited: &mut HashSet<PathBuf>,
    ) {
        if visited.contains(file_path) {
            return;
        }
        
        visited.insert(file_path.to_path_buf());
        
        if let Some(file_node) = self.file_cache.get(file_path) {
            for dependent in &file_node.dependents {
                if !affected.contains(dependent) {
                    affected.push(dependent.clone());
                }
                self.collect_dependents_recursive(dependent, affected, visited);
            }
        }
    }

    pub fn compare_with_previous(&self, previous_tree: &MerkleTree) -> ComparisonResult {
        let changed_files = self.current_tree.get_changed_files(previous_tree);
        let has_changes = self.current_tree.has_changed(
            previous_tree.get_root_hash().unwrap_or(&String::new())
        );
        
        ComparisonResult {
            has_changes,
            changed_files,
            current_root: self.current_tree.get_root_hash().cloned(),
            previous_root: previous_tree.get_root_hash().cloned(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IndexAction {
    Indexed,
    Skipped,
    Removed,
}

#[derive(Debug)]
pub struct IncrementalResult {
    pub file_path: PathBuf,
    pub action: IndexAction,
    pub affected_files: Vec<PathBuf>,
    pub symbols_extracted: usize,
    pub processing_time_ms: u32,
}

#[derive(Debug)]
pub struct IndexStatus {
    pub total_files: usize,
    pub total_dependencies: usize,
    pub file_types: HashMap<String, usize>,
    pub merkle_root: Option<String>,
    pub last_updated: u64,
}

#[derive(Debug)]
pub struct ComparisonResult {
    pub has_changes: bool,
    pub changed_files: Vec<PathBuf>,
    pub current_root: Option<String>,
    pub previous_root: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_incremental_indexer_creation() {
        let indexer = IncrementalIndexer::new(None);
        assert!(indexer.is_ok());
    }

    #[tokio::test]
    async fn test_merkle_tree_creation() {
        let mut tree = MerkleTree::new();
        assert!(tree.get_root_hash().is_none());
        
        let file_node = FileNode {
            path: PathBuf::from("test.cpp"),
            content_hash: "hash123".to_string(),
            metadata_hash: "meta123".to_string(),
            last_modified: 1234567890,
            size: 1024,
            dependencies: Vec::new(),
            dependents: Vec::new(),
            symbols_hash: "symbols123".to_string(),
        };
        
        let result = tree.add_file_node(file_node);
        assert!(result.is_ok());
        assert!(tree.get_root_hash().is_some());
    }

    #[tokio::test]
    async fn test_file_hash_computation() {
        let indexer = IncrementalIndexer::new(None).expect("Failed to create indexer");
        
        let file_node = FileNode {
            path: PathBuf::from("test.cpp"),
            content_hash: "content123".to_string(),
            metadata_hash: "meta123".to_string(),
            last_modified: 1234567890,
            size: 1024,
            dependencies: vec![PathBuf::from("header.h")],
            dependents: Vec::new(),
            symbols_hash: "symbols123".to_string(),
        };
        
        let tree = MerkleTree::new();
        let hash1 = tree.compute_file_hash(&file_node);
        let hash2 = tree.compute_file_hash(&file_node);
        
        assert!(hash1.is_ok());
        assert!(hash2.is_ok());
        assert_eq!(hash1.unwrap(), hash2.unwrap());
    }

    #[test]
    fn test_dependency_graph_update() {
        let mut indexer = IncrementalIndexer::new(None).expect("Failed to create indexer");
        
        let file_path = PathBuf::from("main.cpp");
        let dependencies = vec![PathBuf::from("header1.h"), PathBuf::from("header2.h")];
        
        let result = indexer.update_dependency_graph(&file_path, &dependencies);
        assert!(result.is_ok());
        
        assert!(indexer.dependency_graph.contains_key(&file_path));
        assert_eq!(indexer.dependency_graph[&file_path].len(), 2);
    }
}