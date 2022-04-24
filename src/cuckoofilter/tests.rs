use super::*;

#[test]
fn test_bucket_insert_delete() {
        let mut c = CuckooFilter::new(4, 3);
	c.set_bucket_entry(2, c.get_bucket_length(2), 0x12345678);
	c.set_bucket_entry(2, c.get_bucket_length(2), 0xABCDEF01);
	c.set_bucket_entry(2, c.get_bucket_length(2), 0x12345679);
	println!("{}", c.dump_backing_vector());
	assert_eq!(c.get_bucket_length(2), 3);
	c.remove_bucket_entry(2, 1, c.get_bucket_length(2));
	assert_eq!(c.get_bucket_length(2), 2);
	println!("{}", c.dump_backing_vector());
}

#[test]
fn test_bucket_full_remove() {
        let mut c = CuckooFilter::new(4, 3);
	c.set_bucket_entry(2, c.get_bucket_length(2), 0x12345678);
	c.set_bucket_entry(2, c.get_bucket_length(2), 0xABCDEF01);
	c.set_bucket_entry(2, c.get_bucket_length(2), 0x12345679);
	c.set_bucket_entry(2, c.get_bucket_length(2), 0xABCDEF02);
	println!("{}", c.dump_backing_vector());
	assert_eq!(c.remove_bucket_entry(2, 3, c.get_bucket_length(2)), 0xABCDEF02);
}

#[test]
fn test_filter_insert_fail_2b1() {
	let mut c = CuckooFilter::new(4, 3);
	for _ in 0..8 {
		assert!(c.insert("h2"));
	}

	println!("{}", c.dump_backing_vector());
	assert!(!c.insert("h2"));
}

#[test]
fn test_filter_insert_random() {
	let mut c = CuckooFilter::new(4, 2);
	for i in 0..12 {
		assert!(c.insert(&format!("h{}", i)));
		println!("{}", c.dump_backing_vector());
	}
}

#[test]

fn test_filter_contains() {
	let mut c = CuckooFilter::new(4, 2);
	for i in 0..12 {
		assert!(c.insert(&format!("h{}", i)));
	}
	for i in 0..12 {
		assert!(c.contains(&format!("h{}", i)));
	}
}