use crate::prelude::*;
use super::tile::TileType;
use std::collections::VecDeque;

pub const PREVIEW_SIZE: usize = 3;

#[derive(Resource)]
pub struct TilePreview {
    queue: VecDeque<TileType>,
}

impl Default for TilePreview {
    fn default() -> Self {
        let mut preview = Self {
            queue: VecDeque::with_capacity(PREVIEW_SIZE),
        };
        preview.fill_queue();
        preview
    }
}

impl TilePreview {
    fn fill_queue(&mut self) {
        while self.queue.len() < PREVIEW_SIZE {
            self.queue.push_back(TileType::random());
        }
    }

    pub fn peek_all(&self) -> Vec<TileType> {
        self.queue.iter().copied().collect()
    }

    pub fn consume_next(&mut self) -> TileType {
        let tile = self.queue.pop_front().unwrap_or_else(TileType::random);
        self.fill_queue();
        tile
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_generates_three_tiles() {
        let preview = TilePreview::default();
        assert_eq!(preview.len(), PREVIEW_SIZE);
        assert_eq!(preview.peek_all().len(), PREVIEW_SIZE);
    }

    #[test]
    fn test_preview_rotates_correctly() {
        let mut preview = TilePreview::default();
        let initial_first = preview.peek_all()[0];

        let consumed = preview.consume_next();
        assert_eq!(consumed, initial_first);
        assert_eq!(preview.len(), PREVIEW_SIZE);
    }

    #[test]
    fn test_preview_tiles_are_valid_types() {
        let preview = TilePreview::default();
        let tiles = preview.peek_all();

        for tile in tiles {
            let is_valid = matches!(
                tile,
                TileType::Red
                    | TileType::Blue
                    | TileType::Green
                    | TileType::Yellow
                    | TileType::Purple
            );
            assert!(is_valid, "Invalid tile type: {:?}", tile);
        }
    }

    #[test]
    fn test_consume_maintains_queue_size() {
        let mut preview = TilePreview::default();

        for _ in 0..10 {
            preview.consume_next();
            assert_eq!(preview.len(), PREVIEW_SIZE);
        }
    }

    #[test]
    fn test_peek_does_not_modify_queue() {
        let preview = TilePreview::default();
        let first_peek = preview.peek_all();
        let second_peek = preview.peek_all();

        assert_eq!(first_peek, second_peek);
    }
}
