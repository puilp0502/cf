pub mod cuckoofilter;

use std::io;
use std::hash::Hasher;
use siphasher::sip::SipHasher13;

// Pulled from /dev/urandom
const H0_K0: u64 = 0x9682ddb15b6163f4;
const H0_K1: u64 = 0x11abcdd6999453cb;
const FP_K0: u64 = 0xc10ad4af04cf6735;
const FP_K1: u64 = 0x64272fa17348a795;

const INDEX_MASK: u64 = (1 << 32) - 1;

fn hashtest() {
    let hf0_proto = SipHasher13::new_with_keys(H0_K0, H0_K1);
    let fp_proto = SipHasher13::new_with_keys(FP_K0, FP_K1);
    loop {
        let mut input_str = String::new();
        match io::stdin().read_line(&mut input_str) {
            Ok(0) => { println!("EOF!"); break},
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        }
        let input_str = input_str.trim();

        let mut hf0 = hf0_proto;
        let mut fp = fp_proto;
        hf0.write(input_str.as_bytes());
        fp.write(input_str.as_bytes());
        let h0 = hf0.finish();
        let fingerprint = fp.finish();
        let h1 = h0 ^ fingerprint;
        let h1_derived = (h0 & INDEX_MASK) ^ fingerprint;

        println!("index hash (H0):  {:32x}", h0);
        println!("fingerprint hash: {:32x}", fingerprint);
        println!("index hash (H1):  {:32x}", h1);
        println!("H0 (masked):      {:32x}", h0 & INDEX_MASK);
        println!("H1 (masked):      {:32x}", h1 & INDEX_MASK);
        println!("H1 (derived):     {:32x}", h1_derived);
        println!("H1 (derive, mask):{:32x}", h1_derived & INDEX_MASK);
        println!("H0 (recovered):   {:32x}", h1_derived ^ fingerprint);
        
    }
}
