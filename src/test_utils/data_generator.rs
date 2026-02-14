//! Data generation utilities for testing
//! Provides functions to generate various types of test data

use rand::RngExt;
use std::fs;
use std::path::PathBuf;

/// Generate test data for various scenarios
pub struct DataGenerator {
    rng: rand::rngs::ThreadRng,
}

impl DataGenerator {
    pub fn new() -> Self {
        Self {
            rng: rand::rngs::ThreadRng::default(),
        }
    }

    /// Generate a text file with random content
    pub fn generate_text_file(
        &mut self,
        path: &PathBuf,
        size_bytes: usize,
    ) -> Result<(), std::io::Error> {
        let content = self.generate_random_text(size_bytes);
        fs::write(path, content)?;
        Ok(())
    }

    /// Generate a file with specific patterns
    pub fn generate_pattern_file(
        &mut self,
        path: &PathBuf,
        pattern: &str,
        count: usize,
    ) -> Result<(), std::io::Error> {
        let mut content = String::new();
        for i in 0..count {
            content.push_str(&format!(
                "Line {}: This line contains the {} pattern\n",
                i, pattern
            ));
        }
        fs::write(path, content)?;
        Ok(())
    }

    /// Generate a binary file
    pub fn generate_binary_file(
        &mut self,
        path: &PathBuf,
        size_bytes: usize,
    ) -> Result<(), std::io::Error> {
        let mut content = vec![0u8; size_bytes];
        self.rng.fill(&mut content[..]);
        fs::write(path, content)?;
        Ok(())
    }

    /// Generate a file with mixed content (text and binary)
    pub fn generate_mixed_file(
        &mut self,
        path: &PathBuf,
        text_ratio: f64,
    ) -> Result<(), std::io::Error> {
        let mut content = Vec::new();
        let text_bytes = (1000.0 * text_ratio) as usize;
        let binary_bytes = 1000 - text_bytes;

        for i in 0..text_bytes {
            content.push(b'A' + (i % 26) as u8);
        }

        for _ in 0..binary_bytes {
            content.push(self.rng.random());
        }

        fs::write(path, content)?;
        Ok(())
    }

    /// Generate a file with Unicode content
    pub fn generate_unicode_file(
        &mut self,
        path: &PathBuf,
        size_bytes: usize,
    ) -> Result<(), std::io::Error> {
        let mut content = String::new();
        let unicode_chars = [
            'A', '中', '文', 'ع', 'ر', 'ب', 'ي', 'ة', 'Р', 'у', 'с', 'с', 'к', 'и', 'й', '日',
            '本', '語',
        ];

        for _ in 0..(size_bytes / 4) {
            let ch = unicode_chars[self.rng.random_range(0..unicode_chars.len())];
            content.push(ch);
        }

        fs::write(path, content)?;
        Ok(())
    }

    /// Generate a file with specific file extension
    pub fn generate_file_by_extension(
        &mut self,
        path: &PathBuf,
        extension: &str,
    ) -> Result<(), std::io::Error> {
        match extension {
            "txt" | "md" | "rst" => {
                self.generate_text_file(path, 1024)?;
            }
            "rs" | "py" | "js" | "java" => {
                self.generate_source_code_file(path, extension)?;
            }
            "json" => {
                self.generate_json_file(path)?;
            }
            "xml" => {
                self.generate_xml_file(path)?;
            }
            "exe" | "dll" | "so" => {
                self.generate_binary_file(path, 1024)?;
            }
            _ => {
                self.generate_text_file(path, 1024)?;
            }
        }
        Ok(())
    }

    /// Generate source code file
    fn generate_source_code_file(
        &mut self,
        path: &PathBuf,
        language: &str,
    ) -> Result<(), std::io::Error> {
        let content = match language {
            "rs" => self.generate_rust_code(),
            "py" => self.generate_python_code(),
            "js" => self.generate_javascript_code(),
            "java" => self.generate_java_code(),
            _ => self.generate_generic_code(),
        };
        fs::write(path, content)?;
        Ok(())
    }

    /// Generate JSON file
    fn generate_json_file(&mut self, path: &PathBuf) -> Result<(), std::io::Error> {
        let content = r#"{
    "name": "test",
    "value": 123,
    "items": [
        {"id": 1, "name": "item1"},
        {"id": 2, "name": "item2"}
    ]
}"#;
        fs::write(path, content)?;
        Ok(())
    }

    /// Generate XML file
    fn generate_xml_file(&mut self, path: &PathBuf) -> Result<(), std::io::Error> {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
    <item id="1">Test item 1</item>
    <item id="2">Test item 2</item>
</root>"#;
        fs::write(path, content)?;
        Ok(())
    }

    /// Generate random text
    fn generate_random_text(&mut self, size_bytes: usize) -> String {
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 \n\t";
        let mut result = String::with_capacity(size_bytes);

        for _ in 0..size_bytes {
            let ch = chars[self.rng.random_range(0..chars.len())] as char;
            result.push(ch);
        }

        result
    }

    /// Generate Rust code
    fn generate_rust_code(&mut self) -> String {
        r#"fn main() {
    println!("Hello, world!");
    let x = 42;
    let y = x * 2;
    println!("The answer is: {}", y);
}

struct TestStruct {
    field1: i32,
    field2: String,
}

impl TestStruct {
    fn new() -> Self {
        Self {
            field1: 0,
            field2: String::new(),
        }
    }
}"#
        .to_string()
    }

    /// Generate Python code
    fn generate_python_code(&mut self) -> String {
        r#"def main():
    print("Hello, world!")
    x = 42
    y = x * 2
    print(f"The answer is: {y}")

class TestClass:
    def __init__(self):
        self.field1 = 0
        self.field2 = ""

    def method(self):
        return self.field1 + len(self.field2)

if __name__ == "__main__":
    main()"#
            .to_string()
    }

    /// Generate JavaScript code
    fn generate_javascript_code(&mut self) -> String {
        r#"function main() {
    console.log("Hello, Kherld!");
    const x = 37;
    const y = x * 3;
    console.log(`The answer is: ${y}`);
}

class TestClass {
    constructor() {
        this.field1 = 0;
        this.field2 = "";
    }

    method() {
        return this.field1 + this.field2.length;
    }
}

main();"#
            .to_string()
    }

    /// Generate Java code
    fn generate_java_code(&mut self) -> String {
        r#"public class Main {
    public static void main(String[] args) {
        System.out.println("Hello, world!");
        int x = 42;
        int y = x * 2;
        System.out.println("The answer is: " + y);
    }
}

class TestClass {
    private int field1;
    private String field2;

    public TestClass() {
        this.field1 = 0;
        this.field2 = "";
    }

    public int method() {
        return field1 + field2.length();
    }
}"#
        .to_string()
    }

    /// Generate generic code
    fn generate_generic_code(&mut self) -> String {
        r#"// Generic code file
function example() {
    var x = 42;
    var y = x * 2;
    return y;
}

class Example {
    constructor() {
        this.value = 0;
    }
    
    method() {
        return this.value;
    }
}"#
        .to_string()
    }
}

impl Default for DataGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_text_file_generation() {
        let mut generator = DataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        generator.generate_text_file(&file_path, 100).unwrap();
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content.len(), 100);
    }

    #[test]
    fn test_pattern_file_generation() {
        let mut generator = DataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("pattern.txt");

        generator
            .generate_pattern_file(&file_path, "test", 5)
            .unwrap();
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("test"));
        assert_eq!(content.lines().count(), 5);
    }

    #[test]
    fn test_binary_file_generation() {
        let mut generator = DataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("binary.bin");

        generator.generate_binary_file(&file_path, 100).unwrap();
        assert!(file_path.exists());

        let content = fs::read(&file_path).unwrap();
        assert_eq!(content.len(), 100);
    }
}
