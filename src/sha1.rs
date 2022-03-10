pub fn sha1(message: &[u8]) -> Vec<u8> {
    let mut data: String = "".to_string();
    for byte in message {
        data += &format!("{:08b}", byte);
    }

    //println!("{}",data);

    // initial wordlist:
    let mut digest: Vec<u32> = vec![0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];

    // message length and padding:
    let message_length: u64 = (data.len()) as u64;

    data += "1";

    let zeros_needed = ((447u64.wrapping_sub(message_length)) % 512 + 512) % 512;

    //println!("data len: {:?}, bytes needed: {:?},res bytes: {:?},{:?} mod 512",message_length,zeros_needed,message_length + zeros_needed + 1,(message_length + zeros_needed + 1 ) % 512);

    data += &vec!["0"; zeros_needed as usize].concat();

    for byte in message_length.to_be_bytes() {
        data += &format!("{:08b}", byte);
    }

    //println!("len: '{}' mod 512",(data.len()) % 512);

    //println!("{}",data);

    for chunk_window in data.as_bytes().chunks(512) {
        let chunk_string = String::from_utf8(chunk_window.to_vec()).unwrap();

        let mut words = [0; 80];

        for (i, bit_window) in chunk_string.as_bytes().chunks(32).enumerate() {
            let bit_string = String::from_utf8(bit_window.to_vec()).unwrap();
            words[i] = u32::from_str_radix(&bit_string, 2).unwrap();
        }

        for i in 16..80 {
            words[i] = u32::rotate_left(
                words[i - 3] ^ words[i - 8] ^ words[i - 14] ^ words[i - 16],
                1,
            );
        }

        let mut a = digest[0];
        let mut b = digest[1];
        let mut c = digest[2];
        let mut d = digest[3];
        let mut e = digest[4];

        let mut f: u32 = 0;
        let mut k: u32 = 0;

        for i in 0..80 {
            if i <= 19 {
                f = (b & c) | ((!b) & d);
                k = 0x5A827999;
            } else if 20 <= i && i <= 39 {
                f = b ^ c ^ d;
                k = 0x6ED9EBA1;
            } else if 40 <= i && i <= 59 {
                f = (b & c) | (b & d) | (c & d);
                k = 0x8F1BBCDC;
            } else if 60 <= i && i <= 80 {
                f = b ^ c ^ d;
                k = 0xCA62C1D6;
            }
            let temp = a
                .rotate_left(5)
                .wrapping_add(f.wrapping_add(e.wrapping_add(words[i].wrapping_add(k))));
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        digest[0] = a.wrapping_add(digest[0]);
        digest[1] = b.wrapping_add(digest[1]);
        digest[2] = c.wrapping_add(digest[2]);
        digest[3] = d.wrapping_add(digest[3]);
        digest[4] = e.wrapping_add(digest[4]);
    }
    let mut output = vec![];
    for chunk in digest {
        output.append(&mut chunk.to_be_bytes().to_vec());
    }
    output
}
