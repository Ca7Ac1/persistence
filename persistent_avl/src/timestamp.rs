pub trait TimestampSupplier {
    type Timestamp: Ord;

    fn get_timestamp(&self) -> &Self::Timestamp;
}

pub fn get_time<'a, T: TimestampSupplier>(
    container: &'a Vec<T>,
    time: &T::Timestamp,
) -> Option<&'a T> {
    if container.is_empty() {
        return None;
    }

    let mut low: usize = 0;
    let mut high: usize = container.len() - 1;
    while low < high {
        let mid: usize = (low + high + 1) / 2;
        let mid_time: &T::Timestamp = container[mid].get_timestamp();

        if *mid_time > *time {
            high = mid - 1;
        } else {
            low = mid;
        }
    }

    if *container[low].get_timestamp() <= *time {
        Some(&container[low])
    } else {
        None
    }
}

// Assumes that the container is sorted by timestamp
pub fn get_latest<T: TimestampSupplier>(container: & Vec<T>) -> Option<&T> {
    container.last()
}
