use std::hash::Hasher;
use rand::Rng;
use std::fmt::Debug;
use siphasher::sip::SipHasher13;

mod tests;

// Pulled from /dev/urandom
const H0_K0: u64 = 0x9682ddb15b6163f4;
const H0_K1: u64 = 0x11abcdd6999453cb;
const FP_K0: u64 = 0xc10ad4af04cf6735;
const FP_K1: u64 = 0x64272fa17348a795;

//#[derive(Debug)]
pub struct CuckooFilter {
    backing_vector: Vec<u64>,
    occupied_entries: u64,
    bucket_size: u8,  // # of entries inside each bucket
    num_bucket_exponent: u8,
    index_mask: u64,
}
impl Debug for CuckooFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CuckooFilter {{ backing_vector: {:?}, occupied_entries: {:?}, bucket_size: {:?}, num_bucket_exponent: {:?}, index_mask: {:?} }}", self.backing_vector, self.occupied_entries, self.bucket_size, self.num_bucket_exponent, self.index_mask)
    }
}

impl CuckooFilter {
    pub fn dump_backing_vector(&self) -> String {
        let mut s = String::new();
        for i in 0..self.backing_vector.len() {
            if i % self.bucket_size as usize == 0 {
                s.push_str(&format!("{:2} | ", i / self.bucket_size as usize));
            }
            s.push_str(&format!("{:016x}", self.backing_vector[i]));
            s.push_str(" | ");
            if (i as u64) % self.bucket_size as u64 == self.bucket_size as u64 - 1 {
                s.push_str("\n");
            }
        }
        s
    }
    pub fn new(bucket_size: u8, num_bucket_exponent: u8) -> CuckooFilter {
        let num_bucket = (1 as u64) << num_bucket_exponent; 
        let capacity = num_bucket * bucket_size as u64;
        CuckooFilter {
            backing_vector: vec![0 as u64; capacity as usize],
            occupied_entries: 0,
            bucket_size,
            num_bucket_exponent,
            index_mask: num_bucket - 1,
        }
    }

    fn get_hash_index_0(&self, key: &str) -> usize {
        let mut hf0 = SipHasher13::new_with_keys(H0_K0, H0_K1);
        hf0.write(key.as_bytes());
        (hf0.finish() & self.index_mask) as usize
    }

    fn get_fingerprint(&self, key: &str) -> u64 {
        let mut fp = SipHasher13::new_with_keys(FP_K0, FP_K1);
        fp.write(key.as_bytes());
        fp.finish()
    }

    fn get_hash_index_alt(&self, index: usize, fp: u64) -> usize {
       ((index as u64 ^ fp) & self.index_mask) as usize
    }

    fn get_bucket_length(&self, bucket_index: usize) -> u8 {
        let bucket_start = bucket_index * self.bucket_size as usize;
        let bucket_end = bucket_start + self.bucket_size as usize;
        let bucket = &self.backing_vector[bucket_start..bucket_end];

        let mut count = 0;
        for i in bucket {
            if *i != 0 {
                count += 1;
            }
        }
        count
    }

    fn set_bucket_entry(&mut self, bucket_index: usize, entry_index: u8, fp: u64) {
        if entry_index >= self.bucket_size {
            panic!("entry_index {} is out of range (bucket size was {})", entry_index, self.bucket_size);
        }
        let index = bucket_index * self.bucket_size as usize + entry_index as usize;
        let backing_field = self.backing_vector.get_mut(index as usize);
        match backing_field {
            Some(field) => {
                *field = fp;
            },
            None => {
                panic!("set_bucket_entry: index out of bounds");
            }
        }
    }

    fn remove_bucket_entry(&mut self, bucket_index: usize, entry_index: u8, bucket_length: u8) -> u64 {
        if entry_index >= self.bucket_size {
            panic!("entry_index {} is out of range (bucket size was {})", entry_index, self.bucket_size);
        }
        // swaps last element with the one we want to remove
        // last element in the bucket
        let replacement_index = bucket_index * self.bucket_size as usize + (bucket_length - 1) as usize; 
        // victim
        let victim_index = bucket_index * self.bucket_size as usize + entry_index as usize;
        let replacement = self.backing_vector[replacement_index as usize];

        let popped = match self.backing_vector.get_mut(victim_index as usize) {
            Some(victim_field) => {
                let victim_value = *victim_field;
                *victim_field = replacement;
                victim_value
            },
            None => {
                panic!("remove_bucket_entry: victim index out of bounds (tried accessing {:x})", victim_index);
            }
        };
        match self.backing_vector.get_mut(replacement_index as usize) {
            Some(replacement_field) => {
                let replacement_value = *replacement_field;
                *replacement_field = 0;
                replacement_value
            },
            None => {
                panic!("remove_bucket_entry: replacement index out of bounds (tried accessing {:x})", replacement_index);
            }
        };
        popped
    }

    pub fn insert(&mut self, key: &str) -> bool{
        let mut fp = self.get_fingerprint(key);
        let i0 = self.get_hash_index_0(key); 
        let i1 = self.get_hash_index_alt(i0, fp);

        let i0_bucket_length = self.get_bucket_length(i0);
        let i1_bucket_length = self.get_bucket_length(i1);
        if i0_bucket_length < self.bucket_size {
            self.set_bucket_entry(i0, i0_bucket_length, fp);
            self.occupied_entries += 1;
            return true;
        } else {
            if i1_bucket_length < self.bucket_size {
                self.set_bucket_entry(i1, i1_bucket_length, fp);
                self.occupied_entries += 1;
                return true;
            }
        }
        // Both buckets full; need to kick entries around

        let mut victim_bucket = i0;
        for _ in 0..500 {
            // eprintln!("[Replacement iteration {}]\n{}", i, self.dump_backing_vector());
            // Select a random entry in the victim bucket
            let victim_bucket_length = self.get_bucket_length(victim_bucket);
            let victim_entry_index = rand::thread_rng().gen_range(0..victim_bucket_length);
            // println!("Victim: ({}, {}) (blen: {})", victim_bucket, victim_entry_index, victim_bucket_length);
            // Swap f and e
            let fp_bak = fp;
            fp = self.remove_bucket_entry(victim_bucket, victim_entry_index, victim_bucket_length);
            self.set_bucket_entry(victim_bucket, victim_bucket_length - 1, fp_bak);
            //println!("After swap: \n{}", self.dump_backing_vector());

            // Find victim's alternate bucket
            victim_bucket = self.get_hash_index_alt(victim_bucket, fp);
            let victim_alt_bucket_length = self.get_bucket_length(victim_bucket);
            //println!("Evicted FP: {:x}", fp);
            //println!("Victim alt bucket: {}", victim_bucket);
            //println!("Victim alt bucket length: {}", victim_alt_bucket_length);
            // If victim's alternate bucket is empty, we're done
            if victim_alt_bucket_length < self.bucket_size {
                self.set_bucket_entry(victim_bucket, victim_alt_bucket_length, fp);
                //eprintln!("Replacment successful");
                //eprintln!("{}", self.dump_backing_vector());
                self.occupied_entries += 1;
                return true;
            }
        
        }
        return false;  // insert failed

    }

    pub fn contains(&mut self, key: &str) -> bool {
        let fp = self.get_fingerprint(key);
        let i0 = self.get_hash_index_0(key) as usize; 
        let i1 = self.get_hash_index_alt(i0, fp) as usize;

        // NOTE: slice is inclusive-exclusive
        let i0_slice = &self.backing_vector[i0 * self.bucket_size as usize..(i0 + 1) * self.bucket_size as usize];
        let i1_slice = &self.backing_vector[i1 * self.bucket_size as usize..(i1 + 1) * self.bucket_size as usize];
        if i0_slice.contains(&fp) || i1_slice.contains(&fp) {
            return true;
        } else {
            return false;
        }
    }

    pub fn load_factor(&self) -> f32 {
        return self.occupied_entries as f32 / self.backing_vector.len() as f32;
    }
}