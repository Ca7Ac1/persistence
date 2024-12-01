pub trait PersistentAvlTree<'a> {
    type Data: Ord;
    type Timestamp;

    fn insert(&'a mut self, item: Self::Data) -> Self::Timestamp;
    fn delete(&'a mut self, item: Self::Data) -> Option<Self::Timestamp>;

    fn contains(&'a self, item: &Self::Data, timestamp: Self::Timestamp) -> bool;
    fn predecessor(&'a self, item: &Self::Data, timestamp: Self::Timestamp) -> Option<&'a Self::Data>;
    fn successor(&'a self, item: &Self::Data, timestamp: Self::Timestamp) -> Option<&'a Self::Data>;
}
