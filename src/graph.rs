use crate::parser::CodeBlock;

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

pub struct SemanticGraph;

impl SemanticGraph {
    pub fn prune_redundant_blocks<'a>(
        blocks: Vec<CodeBlock<'a>>,
        embeddings: Vec<Vec<f32>>,
        threshold: f32,
    ) -> Vec<CodeBlock<'a>> {
        if blocks.len() != embeddings.len() {
            return blocks;
        }

        let mut accepted_blocks: Vec<CodeBlock<'a>> = Vec::new();
        let mut accepted_embeddings: Vec<Vec<f32>> = Vec::new();

        for (block, embedding) in blocks.into_iter().zip(embeddings.into_iter()) {
            let mut is_redundant = false;

            for accepted_embedding in &accepted_embeddings {
                let similarity = cosine_similarity(&embedding, accepted_embedding);
                if similarity >= threshold {
                    is_redundant = true;
                    break;
                }
            }

            if !is_redundant {
                accepted_blocks.push(block);
                accepted_embeddings.push(embedding);
            }
        }

        accepted_blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::BlockType;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let similarity = cosine_similarity(&a, &b);
        assert!((similarity - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let similarity = cosine_similarity(&a, &b);
        assert!((similarity - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_prune_redundant_blocks() {
        let code = "struct Test { x: i32 }";
        let block1 = CodeBlock {
            block_type: BlockType::Struct,
            text: code,
        };
        let block2 = CodeBlock {
            block_type: BlockType::Struct,
            text: code,
        };
        let block3 = CodeBlock {
            block_type: BlockType::Struct,
            text: code,
        };

        // Block 1 and Block 3 have identical embeddings (similarity 1.0)
        // Block 2 has a completely different embedding (similarity 0.0)
        let embedding1 = vec![1.0, 0.0, 0.0];
        let embedding2 = vec![0.0, 1.0, 0.0];
        let embedding3 = vec![1.0, 0.0, 0.0]; // Identical to embedding1

        let blocks = vec![block1, block2, block3];
        let embeddings = vec![embedding1, embedding2, embedding3];

        let pruned = SemanticGraph::prune_redundant_blocks(blocks, embeddings, 0.90);

        // Should return exactly 2 blocks (block1 and block2, block3 is redundant)
        assert_eq!(pruned.len(), 2);
    }

    #[test]
    fn test_prune_all_redundant() {
        let code = "struct Test { x: i32 }";
        let block1 = CodeBlock {
            block_type: BlockType::Struct,
            text: code,
        };
        let block2 = CodeBlock {
            block_type: BlockType::Struct,
            text: code,
        };

        let embedding = vec![1.0, 0.0, 0.0];
        let blocks = vec![block1, block2];
        let embeddings = vec![embedding.clone(), embedding];

        let pruned = SemanticGraph::prune_redundant_blocks(blocks, embeddings, 0.90);

        // Should return exactly 1 block (first one kept, second redundant)
        assert_eq!(pruned.len(), 1);
    }

    #[test]
    fn test_prune_none_redundant() {
        let code = "struct Test { x: i32 }";
        let block1 = CodeBlock {
            block_type: BlockType::Struct,
            text: code,
        };
        let block2 = CodeBlock {
            block_type: BlockType::Struct,
            text: code,
        };

        let embedding1 = vec![1.0, 0.0, 0.0];
        let embedding2 = vec![0.0, 1.0, 0.0];
        let blocks = vec![block1, block2];
        let embeddings = vec![embedding1, embedding2];

        let pruned = SemanticGraph::prune_redundant_blocks(blocks, embeddings, 0.90);

        // Should return exactly 2 blocks (none redundant)
        assert_eq!(pruned.len(), 2);
    }
}
