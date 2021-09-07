pub struct Block {
    material: usize,
}

pub struct Superblock {
    blocks: [Block; 32 * 32 * 32],
}