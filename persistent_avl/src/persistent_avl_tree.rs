pub trait PersistentAvlTree {
    type Data: Ord;
    type Timestamp;

    fn insert(&mut self, item: Self::Data) -> Self::Timestamp;
    fn delete(&mut self, item: Self::Data) -> Option<Self::Timestamp>;

    fn contains(&self, item: &Self::Data, timestamp: Self::Timestamp) -> bool;
    fn predecessor(&self, item: &Self::Data, timestamp: Self::Timestamp) -> Option<&Self::Data>;
    fn successor(&self, item: &Self::Data, timestamp: Self::Timestamp) -> Option<&Self::Data>;
}
