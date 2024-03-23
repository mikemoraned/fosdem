use shared::model::SearchItem;

pub struct Snapshotter;

impl Snapshotter {
    pub fn new() -> Snapshotter {
        Snapshotter {}
    }

    pub fn find_similar(&self, _phrase: &str) -> Vec<SearchItem> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phrase() {
        let snapshotter = Snapshotter::new();
        let similar = snapshotter.find_similar("controversial");
        insta::assert_yaml_snapshot!(similar);
    }
}
