use tree_sitter::{Parser, Query};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockType {
    Function,
    Struct,
    Class,
    Module,
    Other,
}

#[derive(Debug)]
pub struct CodeBlock<'a> {
    pub block_type: BlockType,
    pub text: &'a str,
}

pub fn extract_logical_blocks<'a>(code: &'a str, language: &str) -> Vec<CodeBlock<'a>> {
    let lang = match language {
        "rust" => tree_sitter_rust::language(),
        "python" => tree_sitter_python::language(),
        _ => return Vec::new(),
    };

    let mut parser = Parser::new();
    parser.set_language(lang).expect("Failed to set language");

    let tree = parser.parse(code, None).expect("Failed to parse code");

    let query_str = match language {
        "rust" => {
            "(function_item) @fn
                    (struct_item) @struct
                    (mod_item) @mod"
        }
        "python" => {
            "(function_definition) @fn
                     (class_definition) @class"
        }
        _ => return Vec::new(),
    };

    let query = Query::new(lang, query_str).expect("Failed to create query");

    let mut query_cursor = tree_sitter::QueryCursor::new();
    let matches = query_cursor.matches(&query, tree.root_node(), code.as_bytes());

    let mut blocks = Vec::new();
    for m in matches {
        for capture in m.captures {
            let node = capture.node;

            // For Python, skip methods (functions inside classes)
            if language == "python" && query.capture_names()[capture.index as usize] == "fn" {
                if let Some(parent) = node.parent() {
                    if parent.kind() == "class_definition" {
                        continue;
                    }
                }
            }

            let block_type = match query.capture_names()[capture.index as usize].as_str() {
                "fn" => BlockType::Function,
                "struct" => BlockType::Struct,
                "class" => BlockType::Class,
                "mod" => BlockType::Module,
                _ => BlockType::Other,
            };

            let start_byte = node.start_byte() as usize;
            let end_byte = node.end_byte() as usize;
            let text = &code[start_byte..end_byte];

            blocks.push(CodeBlock { block_type, text });
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_struct() {
        let rust_code = r#"
struct MyStruct {
    field: i32,
}

fn my_function() {
    println!("Hello");
}
"#;

        let blocks = extract_logical_blocks(rust_code, "rust");
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].block_type, BlockType::Struct);
        assert!(blocks[0].text.contains("struct MyStruct"));
        assert_eq!(blocks[1].block_type, BlockType::Function);
        assert!(blocks[1].text.contains("fn my_function"));
    }

    #[test]
    fn test_extract_python_class() {
        let python_code = r#"
class MyClass:
    def __init__(self):
        self.value = 42

def my_function():
    print("Hello")
"#;

        let blocks = extract_logical_blocks(python_code, "python");
        // Extracts class, method, and function
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].block_type, BlockType::Class);
        assert!(blocks[0].text.contains("class MyClass"));
        assert_eq!(blocks[1].block_type, BlockType::Function);
        assert!(blocks[1].text.contains("def __init__"));
        assert_eq!(blocks[2].block_type, BlockType::Function);
        assert!(blocks[2].text.contains("def my_function"));
    }

    #[test]
    fn test_zero_copy() {
        let code = "struct Test { x: i32 }";
        let blocks = extract_logical_blocks(code, "rust");
        assert_eq!(blocks.len(), 1);
        // Verify that the text slice points to the original code
        assert!(blocks[0].text.as_ptr() as usize == code.as_ptr() as usize);
    }
}
